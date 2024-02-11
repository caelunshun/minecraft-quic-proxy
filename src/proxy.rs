//! Implements proxy logic.

use crate::{
    packet_translation::{PacketTranslator, TranslatePacket},
    protocol::{
        packet,
        packet::{side, state, state::Play, ProtocolState},
        vanilla_codec::{CompressionThreshold, EncryptionKey, VanillaCodec},
    },
    sequence::SequencesHandle,
    stream::{RecvStreamHandle, SendStreamHandle},
    stream_allocation::{AllocateStream, Allocation, StreamAllocator},
};
use anyhow::{bail, Context};
use quinn::Connection;
use std::{any::type_name, marker::PhantomData, ops::ControlFlow, sync::Arc};
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
            if bytes_read == 0 {
                bail!("disconnected from TCP");
            }
            codec.give_data(&mut buffer[..bytes_read]);
        }
    }
}

/// Utility to listen for packets on all incoming
/// QUIC streams (unidirectional only).
struct QuicReceiver<Side: packet::Side, State: ProtocolState> {
    connection: Connection,
    stream_receives_tx: flume::Sender<anyhow::Result<Side::RecvPacket<State>>>,
    stream_receives: flume::Receiver<anyhow::Result<Side::RecvPacket<State>>>,
}

impl<Side, State> QuicReceiver<Side, State>
where
    Side: packet::Side,
    State: ProtocolState,
{
    pub fn new(connection: Connection) -> Self {
        let (stream_receives_tx, stream_receives) = flume::bounded(16);
        Self {
            connection,
            stream_receives,
            stream_receives_tx,
        }
    }

    pub async fn recv_packet(&self) -> anyhow::Result<Side::RecvPacket<State>> {
        loop {
            select! {
                packet = self.stream_receives.recv_async() => {
                    return packet?;
                }
                new_stream = RecvStreamHandle::<Side, State>::accept(&self.connection, "incoming_any") => {
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

/// `PacketIo` over QUIC, using only one stream
/// for all packets. This is used in the Handshake/Login/Status
/// /Configuration states.
///
/// Only one receive stream will be accepted. Others are ignored.
/// (This ensures that state switching works correctly.)
pub struct SingleQuicPacketIo<Side: packet::Side, State: ProtocolState> {
    connection: Connection,
    send_stream: SendStreamHandle<Side, State>,
    recv_stream: Mutex<Option<RecvStreamHandle<Side, State>>>,
}

impl<Side, State> SingleQuicPacketIo<Side, State>
where
    Side: packet::Side,
    State: ProtocolState,
{
    pub async fn new(connection: &Connection) -> anyhow::Result<Self> {
        Ok(Self {
            connection: connection.clone(),
            send_stream: SendStreamHandle::open(connection, type_name::<State>()).await?,
            recv_stream: Mutex::new(None),
        })
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// Changes to a new protocol state.
    ///
    /// All current streams are dropped. Both the client and gateway
    /// sides of the connection must make the state change at the same
    /// time, so that stream cooperation works correctly.
    pub async fn switch_state<NewState: ProtocolState>(
        self,
    ) -> anyhow::Result<SingleQuicPacketIo<Side, NewState>> {
        SingleQuicPacketIo::new(&self.connection).await
    }
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
        loop {
            let mut recv_stream = self.recv_stream.lock().await;

            match &mut *recv_stream {
                Some(stream) => {
                    return stream
                        .recv_packet()
                        .await
                        .map(|opt| opt.context("end of stream"))?
                }
                None => {
                    *recv_stream = Some(
                        RecvStreamHandle::accept(&self.connection, type_name::<State>()).await?,
                    );
                }
            }
        }
    }
}

/// `PacketIo` over QUIC, using full stream and datagram/sequence
/// allocation.
///
/// Only valid for `state::Play`.
pub struct QuicPacketIo<Side: packet::Side> {
    connection: Connection,
    stream_allocator: Mutex<StreamAllocator<Side>>,
    packet_translator: Mutex<PacketTranslator>,
    receiver: QuicReceiver<Side, state::Play>,
    sequences: SequencesHandle<Side>,
}

impl<Side> QuicPacketIo<Side>
where
    Side: packet::Side,
{
    pub async fn new(connection: Connection) -> anyhow::Result<Self> {
        Ok(Self {
            stream_allocator: Mutex::new(StreamAllocator::new(&connection).await?),
            packet_translator: Mutex::new(PacketTranslator::new()),
            sequences: SequencesHandle::new(connection.clone()),
            receiver: QuicReceiver::new(connection.clone()),
            connection,
        })
    }
}

impl<Side> PacketIo<Side, state::Play> for QuicPacketIo<Side>
where
    Side: packet::Side,
    StreamAllocator<Side>: AllocateStream<Side>,
    PacketTranslator: TranslatePacket<Side>,
{
    async fn send_packet(&self, packet: Side::SendPacket<Play>) -> anyhow::Result<()> {
        let packet = self
            .packet_translator
            .lock()
            .await
            .translate_packet(&packet)
            .unwrap_or(packet);

        let mut stream_allocator = self.stream_allocator.lock().await;
        let allocation = stream_allocator.allocate_stream_for(&packet).await?;
        drop(stream_allocator);

        match allocation {
            Allocation::Stream(stream) => stream.send_packet(packet).await,
            Allocation::UnreliableSequence(key) => self.sequences.send_packet(key, packet).await,
        }
    }

    async fn recv_packet(&self) -> anyhow::Result<Side::RecvPacket<Play>> {
        select! {
            packet = self.sequences.recv_packet() => packet,
            packet = self.receiver.recv_packet() => packet,
        }
    }
}

/// Utility to proxy packets between two `PacketIo` instances.
pub struct Proxy<Client, Server, State> {
    pending_tasks: JoinSet<anyhow::Result<()>>,
    client: Arc<Client>,
    server: Arc<Server>,
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
            client: Arc::new(client),
            server: Arc::new(server),
            _marker: PhantomData,
        }
    }

    pub fn client_mut(&mut self) -> &mut Client {
        Arc::get_mut(&mut self.client).unwrap()
    }

    pub fn server_mut(&mut self) -> &mut Server {
        Arc::get_mut(&mut self.server).unwrap()
    }

    /// Proxies packets between the two endpoints.
    ///
    /// Returns once either
    /// * an error or disconnect occurs; or
    /// * one of the provided callbacks returns `ControlFlow::Break`.
    pub async fn run<R>(
        &mut self,
        mut intercept_client_packet: impl FnMut(
            &mut <side::Client as packet::Side>::SendPacket<State>,
        ) -> ControlFlow<R>,
        mut intercept_server_packet: impl FnMut(
            &mut <side::Server as packet::Side>::SendPacket<State>,
        ) -> ControlFlow<R>,
    ) -> anyhow::Result<R> {
        let result = loop {
            select! {
                client_packet = self.client.recv_packet() => {
                    let mut client_packet= client_packet?;
                    let control_flow = intercept_client_packet(&mut client_packet);

                    tracing::debug!("client => server: {}", client_packet.as_ref());
                    let server = Arc::clone(&self.server);
                    self.pending_tasks.spawn_local(async move {
                        server.send_packet(client_packet).await
                    });

                    if let ControlFlow::Break(result) = control_flow{
                        break Ok(result);
                    }
                }
                server_packet = self.server.recv_packet() => {
                    let mut server_packet = server_packet?;
                    let control_flow = intercept_server_packet(&mut server_packet);

                    tracing::debug!("server => client: {}", server_packet.as_ref());
                    let client = Arc::clone(&self.client);
                    self.pending_tasks.spawn_local(async move {
                       client.send_packet(server_packet).await
                    });

                    if let ControlFlow::Break(result) = control_flow {
                        break Ok(result );
                    }
                }
                opt_result = self.pending_tasks.join_next(), if !self.pending_tasks.is_empty() => {
                    opt_result.expect("no task?")??;
                }
            }
        };

        while let Some(result) = self.pending_tasks.join_next().await {
            result??;
        }

        result
    }

    pub fn into_parts(self) -> (Client, Server) {
        (
            Arc::into_inner(self.client).unwrap(),
            Arc::into_inner(self.server).unwrap(),
        )
    }
}
