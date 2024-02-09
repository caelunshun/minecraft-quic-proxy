//! Implements proxy logic.

use crate::{
    protocol::{
        packet,
        packet::{side, state, state::Play, ProtocolState},
        vanilla_codec::{CompressionThreshold, EncryptionKey, VanillaCodec},
    },
    sequence::SequencesHandle,
    stream::{RecvStreamHandle, SendStreamHandle},
    stream_allocation::{AllocateStream, Allocation, StreamAllocator},
};
use anyhow::{anyhow, Context};
use quinn::Connection;
use std::{marker::PhantomData, ops::ControlFlow, rc::Rc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    select,
    sync::Mutex,
    task,
    task::JoinSet,
};

pub trait PacketIo<Side: packet::Side, State: ProtocolState> {
    async fn send_packet(&self, packet: Side::SendPacket<State>) -> anyhow::Result<()>;

    /// _Must_ be cancellation-safe: if this future
    /// is cancelled, no received packet can be dropped.
    /// (This is required so that the proxy can call
    /// this future in a `select!` loop.)
    async fn recv_packet(&self) -> anyhow::Result<Side::RecvPacket<State>>;
}

/// `PacketIo` over vanilla TCP.
pub struct VanillaPacketIo<Side: packet::Side, State: ProtocolState> {
    send_stream: Mutex<OwnedWriteHalf>,
    recv_stream: Mutex<OwnedReadHalf>,
    send_codec: Mutex<VanillaCodec<Side, State>>,
    recv_codec: Mutex<VanillaCodec<Side, State>>,
}

impl<Side, State> VanillaPacketIo<Side, State>
where
    Side: packet::Side,
    State: ProtocolState,
{
    pub fn new(stream: TcpStream) -> anyhow::Result<Self> {
        let (recv_stream, send_stream) = stream.into_split();
        Ok(Self {
            send_stream: Mutex::new(send_stream),
            recv_stream: Mutex::new(recv_stream),
            send_codec: Mutex::new(VanillaCodec::new()),
            recv_codec: Mutex::new(VanillaCodec::new()),
        })
    }

    pub fn enable_compression(&mut self, threshold: CompressionThreshold) {
        self.send_codec.get_mut().enable_compression(threshold);
        self.recv_codec.get_mut().enable_compression(threshold);
    }

    pub fn enable_encryption(&mut self, key: EncryptionKey) {
        self.send_codec.get_mut().enable_encryption(key);
        self.recv_codec.get_mut().enable_encryption(key);
    }

    pub fn switch_state<NewState: ProtocolState>(self) -> VanillaPacketIo<Side, NewState> {
        VanillaPacketIo {
            send_stream: self.send_stream,
            recv_stream: self.recv_stream,
            send_codec: Mutex::new(self.send_codec.into_inner().switch_state()),
            recv_codec: Mutex::new(self.recv_codec.into_inner().switch_state()),
        }
    }
}

impl<Side, State> PacketIo<Side, State> for VanillaPacketIo<Side, State>
where
    Side: packet::Side,
    State: ProtocolState,
{
    async fn send_packet(&self, packet: Side::SendPacket<State>) -> anyhow::Result<()> {
        let bytes = {
            let mut codec = self.send_codec.lock().await;
            codec.encode_packet(&packet)?
        };
        let mut stream = self.send_stream.lock().await;
        stream.write_all(&bytes).await?;
        Ok(())
    }

    async fn recv_packet(&self) -> anyhow::Result<Side::RecvPacket<State>> {
        let mut buffer = [0u8; 256];
        loop {
            // Both locks must occur here to ensure cancellation safety
            let mut codec = self.recv_codec.lock().await;
            let mut stream = self.recv_stream.lock().await;

            if let Some(packet) = codec.decode_packet()? {
                return Ok(packet);
            }

            let bytes_read = stream.read(&mut buffer).await?;
            codec.give_data(&mut buffer[..bytes_read]);
        }
    }
}

/// `PacketIo` over QUIC, using only one stream
/// for all packets. This is used in the Handshake/Login/Status
/// /Configuration states.
pub struct SingleQuicPacketIo<Side: packet::Side, State: ProtocolState> {
    send_stream: SendStreamHandle<Side, State>,
    recv_stream: RecvStreamHandle<Side, State>,
}

impl<Side, State> PacketIo<Side, State> for SingleQuicPacketIo<Side, State>
where
    Side: packet::Side,
    State: ProtocolState,
{
    async fn send_packet(&self, packet: Side::SendPacket<State>) -> anyhow::Result<()> {
        self.send_stream.send_packet(packet).await
    }

    async fn recv_packet(&self) -> anyhow::Result<Side::RecvPacket<State>> {
        self.recv_stream
            .recv_packet()
            .await
            .map(|opt| opt.ok_or_else(|| anyhow!("end of stream")))?
    }
}

/// `PacketIo` over QUIC, using full stream and datagram/sequence
/// allocation.
///
/// Only valid for `state::Play`.
pub struct QuicPacketIo<Side: packet::Side> {
    connection: Connection,
    stream_allocator: Mutex<StreamAllocator<Side>>,
    stream_receives_tx: flume::Sender<anyhow::Result<Side::RecvPacket<Play>>>,
    stream_receives: flume::Receiver<anyhow::Result<Side::RecvPacket<Play>>>,
    sequences: SequencesHandle<Side>,
}

impl<Side> QuicPacketIo<Side>
where
    Side: packet::Side,
{
    pub async fn new(connection: Connection) -> anyhow::Result<Self> {
        let (stream_receives_tx, stream_receives) = flume::bounded(16);
        Ok(Self {
            stream_allocator: Mutex::new(StreamAllocator::new(&connection).await?),
            sequences: SequencesHandle::new(connection.clone()),
            connection,
            stream_receives,
            stream_receives_tx,
        })
    }
}

impl<Side> PacketIo<Side, state::Play> for QuicPacketIo<Side>
where
    Side: packet::Side,
    StreamAllocator<Side>: AllocateStream<Side>,
{
    async fn send_packet(&self, packet: Side::SendPacket<Play>) -> anyhow::Result<()> {
        let mut stream_allocator = self.stream_allocator.lock().await;
        let allocation = stream_allocator.allocate_stream_for(&packet).await?;
        drop(stream_allocator);

        match allocation {
            Allocation::Stream(stream) => stream.send_packet(packet).await,
            Allocation::Sequence(key) => self.sequences.send_packet(key, packet).await,
        }
    }

    async fn recv_packet(&self) -> anyhow::Result<Side::RecvPacket<Play>> {
        loop {
            select! {
                packet = self.sequences.recv_packet() => return packet,
                packet = self.stream_receives.recv_async() => return packet.context("disconnected")?,
                new_stream = RecvStreamHandle::<Side, state::Play>::accept(&self.connection) => {
                    let new_stream = new_stream?;
                    let stream_receives = self.stream_receives_tx.clone();
                    task::spawn(async move {
                        loop {
                            match new_stream.recv_packet().await {
                                Ok(Some(packet)) => if stream_receives.send_async(Ok(packet)).await.is_err() {
                                    break;
                                }
                                Ok(None) => break,
                                Err(e) => {
                                    stream_receives.send_async(Err(e)).await.ok();
                                    break;
                                }
                            }
                        }
                    });
                }
            }
        }
    }
}

/// Utility to proxy packets between two `PacketIo` instances.
///
/// Must be in a `LocalSet` context.
pub struct Proxy<Client, Server, State> {
    pending_tasks: JoinSet<anyhow::Result<()>>,
    client: Rc<Client>,
    server: Rc<Server>,
    _marker: PhantomData<State>,
}

impl<Client, Server, State> Proxy<Client, Server, State>
where
    Client: PacketIo<side::Server, State> + 'static,
    Server: PacketIo<side::Client, State> + 'static,
    State: ProtocolState,
{
    pub fn new(client: Client, server: Server) -> Self {
        Self {
            pending_tasks: JoinSet::new(),
            client: Rc::new(client),
            server: Rc::new(server),
            _marker: PhantomData,
        }
    }

    /// Proxies packets between the two endpoints.
    ///
    /// Returns once either
    /// * an error or disconnect occurs; or
    /// * one of the provided callbacks returns `ControlFlow::Break`.
    pub async fn proxy(
        &mut self,
        mut intercept_client_packet: impl FnMut(
            &mut <side::Client as packet::Side>::SendPacket<State>,
        ) -> ControlFlow<()>,
        mut intercept_server_packet: impl FnMut(
            &mut <side::Server as packet::Side>::SendPacket<State>,
        ) -> ControlFlow<()>,
    ) -> anyhow::Result<()> {
        loop {
            select! {
                client_packet = self.client.recv_packet() => {
                    let mut client_packet= client_packet?;
                    let control_flow = intercept_client_packet(&mut client_packet);

                    let server = Rc::clone(&self.server);
                    self.pending_tasks.spawn_local(async move {
                        server.send_packet(client_packet).await
                    });

                    if control_flow.is_break() {
                        break;
                    }
                }
                server_packet = self.server.recv_packet() => {
                    let mut server_packet = server_packet?;
                    let control_flow = intercept_server_packet(&mut server_packet);

                    let client = Rc::clone(&self.client);
                    self.pending_tasks.spawn_local(async move {
                       client.send_packet(server_packet).await
                    });

                    if control_flow.is_break() {
                        break;
                    }
                }
                opt_result = self.pending_tasks.join_next(), if !self.pending_tasks.is_empty() => {
                    opt_result.expect("no task?")??;
                }
            }
            todo!()
        }

        while let Some(result) = self.pending_tasks.join_next().await {
            result??;
        }

        Ok(())
    }

    pub fn into_parts(self) -> (Client, Server) {
        (
            Rc::into_inner(self.client).unwrap(),
            Rc::into_inner(self.server).unwrap(),
        )
    }
}
