use crate::protocol::{optimized_codec::OptimizedCodec, packet, packet::ProtocolState};
use anyhow::anyhow;
use quinn::Connection;
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
    pub async fn open(connection: &Connection) -> anyhow::Result<Self> {
        let mut stream = connection.open_uni().await?;
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
        });
        Ok(Self { send_data: sender })
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
    pub async fn accept(connection: &Connection) -> anyhow::Result<Self> {
        let mut stream = connection.accept_uni().await?;
        let (sender, receiver) = flume::bounded::<anyhow::Result<Side::RecvPacket<State>>>(4);

        task::spawn(async move {
            let mut codec = OptimizedCodec::<Side, State>::new();
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
        });

        Ok(Self {
            recv_data: receiver,
        })
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
