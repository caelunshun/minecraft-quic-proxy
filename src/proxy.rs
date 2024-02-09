//! Implements proxy logic.

use crate::{
    protocol::{
        packet,
        packet::{state, state::Play, ProtocolState},
    },
    sequence::SequencesHandle,
    stream::{RecvStreamHandle, SendStreamHandle},
    stream_allocation::{AllocateStream, Allocation, StreamAllocator},
};
use anyhow::{anyhow, Context};
use tokio::{
    net::TcpStream,
    select,
    sync::{oneshot, Mutex},
};

pub trait PacketIo<Side: packet::Side, State: ProtocolState> {
    async fn send_packet(&self, packet: Side::SendPacket<State>) -> anyhow::Result<()>;

    async fn recv_packet(&self) -> anyhow::Result<Side::RecvPacket<State>>;
}

type SendPacket<Side, State> = (
    <Side as packet::Side>::SendPacket<State>,
    oneshot::Sender<anyhow::Result<()>>,
);

/// `PacketIo` over vanilla TCP.
pub struct VanillaPacketIo<Side: packet::Side, State: ProtocolState> {
    sender: flume::Sender<SendPacket<Side, State>>,
    receiver: flume::Receiver<anyhow::Result<Side::RecvPacket<State>>>,
}

impl<Side, State> PacketIo<Side, State> for VanillaPacketIo<Side, State>
where
    Side: packet::Side,
    State: ProtocolState,
{
    pub fn new(stream: TcpStream) -> anyhow::Result<Self> {}

    async fn send_packet(&self, packet: Side::SendPacket<State>) -> anyhow::Result<()> {
        let (completion_tx, completion_rx) = oneshot::channel();
        self.sender
            .send_async((packet, completion_tx))
            .await
            .ok()
            .context("disconnected")?;
        completion_rx.await.context("stream died")?
    }

    async fn recv_packet(&self) -> anyhow::Result<Side::RecvPacket<State>> {
        self.receiver.recv_async().await.context("disconnected")?
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
    stream_allocator: Mutex<StreamAllocator<Side>>,
    sequences: SequencesHandle<Side>,
}

impl<Side> PacketIo<Side, state::Play> for QuicPacketIo<Side>
where
    Side: packet::Side + 'static,
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
        select! {
            packet = self.sequences.recv_packet() => packet,
        }
    }
}
