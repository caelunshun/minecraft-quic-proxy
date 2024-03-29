use crate::protocol::{optimized_codec::OptimizedCodec, packet, packet::ProtocolState};
use anyhow::anyhow;
use quinn::{Connection, RecvStream, SendStream};
use std::borrow::Cow;
use tokio::{sync::oneshot, task};

type SendPacket<Side, State> = (
    <Side as packet::Side>::SendPacket<State>,
    oneshot::Sender<anyhow::Result<()>>,
);

/// An open sending QUIC stream.
///
/// This combines a `quinn::SendStream` with the codec
/// state used to delimit packets. Data sends are offloaded
/// to a Tokio task.
#[derive(Clone)]
pub struct SendStreamHandle<Side: packet::Side, State: ProtocolState> {
    send_data: flume::Sender<SendPacket<Side, State>>,
}

impl<Side, State> SendStreamHandle<Side, State>
where
    Side: packet::Side,
    State: ProtocolState,
{
    /// Opens a new stream.
    pub async fn open(
        connection: &Connection,
        name: impl Into<Cow<'static, str>>,
        priority: i32,
    ) -> anyhow::Result<Self> {
        let stream = connection.open_uni().await?;
        stream.set_priority(priority)?;
        Ok(Self::from_stream(stream, name))
    }

    fn from_stream(mut stream: SendStream, name: impl Into<Cow<'static, str>>) -> Self {
        let name = name.into();
        let (sender, receiver) = flume::bounded::<SendPacket<Side, State>>(4);
        task::spawn(async move {
            let mut codec = OptimizedCodec::<Side, State>::new();
            while let Ok((packet, completion)) = receiver.recv_async().await {
                let data = codec.encode_packet(&packet).expect("encoding failed");
                let result = stream.write_all(&data).await;
                let errored = result.is_err();
                completion.send(result.map_err(anyhow::Error::from)).ok();
                if errored {
                    break;
                }
            }
            let id = stream.id();
            tracing::trace!("Closing send stream {name} (QUIC ID = {id:?})");
        });
        Self { send_data: sender }
    }

    /// Sends a packet on this stream.
    pub async fn send_packet(&self, packet: Side::SendPacket<State>) -> anyhow::Result<()> {
        let (completion_tx, completion_rx) = oneshot::channel();
        self.send_data
            .send_async((packet, completion_tx))
            .await
            .ok();
        completion_rx.await.map_err(|_| anyhow!("stream dead"))?
    }
}

/// An open receiving QUIC stream.
///
/// This combines a `quinn::RecvStream` with the codec
/// needed to delimit packets. Data receiving is offloaded
/// to a separate task (with backpressure).
#[derive(Clone)]
pub struct RecvStreamHandle<Side: packet::Side, State: ProtocolState> {
    recv_data: flume::Receiver<anyhow::Result<Side::RecvPacket<State>>>,
}

impl<Side, State> RecvStreamHandle<Side, State>
where
    Side: packet::Side,
    State: ProtocolState,
{
    /// Accepts the next stream on the connection.
    pub async fn accept(
        connection: &Connection,
        name: impl Into<Cow<'static, str>>,
    ) -> anyhow::Result<Self> {
        let stream = connection.accept_uni().await?;
        Ok(Self::from_stream(stream, name))
    }

    fn from_stream(mut stream: RecvStream, name: impl Into<Cow<'static, str>>) -> Self {
        let name = name.into();
        let (sender, receiver) = flume::bounded::<anyhow::Result<Side::RecvPacket<State>>>(4);

        task::spawn(async move {
            let mut codec = OptimizedCodec::<Side, State>::new();
            let id = stream.id();
            drive_recv_stream(&mut stream, &mut codec, sender).await;
            tracing::trace!("Lost receive stream {name} (QUIC ID = {id:?})");
        });

        Self {
            recv_data: receiver,
        }
    }

    /// Waits for the next packet to be received on this stream.
    /// Returns `None` if the stream was closed and there are no more packets.
    pub async fn recv_packet(&self) -> anyhow::Result<Option<Side::RecvPacket<State>>> {
        match self.recv_data.recv_async().await {
            Ok(Ok(packet)) => Ok(Some(packet)),
            Ok(Err(e)) => Err(e),
            Err(_) => Ok(None),
        }
    }
}

async fn drive_recv_stream<Side: packet::Side, State: ProtocolState>(
    stream: &mut RecvStream,
    codec: &mut OptimizedCodec<Side, State>,
    sender: flume::Sender<anyhow::Result<Side::RecvPacket<State>>>,
) {
    let mut buffer = [0u8; 256];
    loop {
        loop {
            match codec.decode_packet() {
                Ok(Some(packet)) => {
                    if sender.send_async(Ok(packet)).await.is_err() {
                        return;
                    }
                }
                Ok(None) => {
                    break;
                }
                Err(e) => {
                    sender.send_async(Err(e)).await.ok();
                    return;
                }
            }
        }

        match stream.read(&mut buffer).await {
            Ok(Some(bytes_read)) => {
                codec.give_data(&buffer[..bytes_read]);
            }
            Ok(None) => break,
            Err(e) => {
                sender.send_async(Err(e.into())).await.ok();
                break;
            }
        }
    }
}

pub async fn accept_bi<Side, State>(
    connection: &Connection,
    name: impl Into<Cow<'static, str>>,
) -> anyhow::Result<(SendStreamHandle<Side, State>, RecvStreamHandle<Side, State>)>
where
    Side: packet::Side,
    State: ProtocolState,
{
    let name = name.into();
    let (send, recv) = connection.accept_bi().await?;
    Ok((
        SendStreamHandle::from_stream(send, name.clone()),
        RecvStreamHandle::from_stream(recv, name),
    ))
}

pub async fn open_bi<Side, State>(
    connection: &Connection,
    name: impl Into<Cow<'static, str>>,
) -> anyhow::Result<(SendStreamHandle<Side, State>, RecvStreamHandle<Side, State>)>
where
    Side: packet::Side,
    State: ProtocolState,
{
    let name = name.into();
    let (send, recv) = connection.open_bi().await?;
    Ok((
        SendStreamHandle::from_stream(send, name.clone()),
        RecvStreamHandle::from_stream(recv, name),
    ))
}
