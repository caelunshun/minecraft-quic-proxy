//! The control stream is the first stream opened over the QUIC
//! connection. It is always bidirectional.
//!
//! This stream contains special messages used by the proxy system.
//! It uses `bincode` for encoding and a simple length-delimited codec
//! for packet framing. It is not related to the Minecraft protocol encoding.

use crate::io_duplex::IoDuplex;
use anyhow::{anyhow, Context};
use bincode::Options;
use futures::{SinkExt, StreamExt};
use quinn::{Connection, RecvStream, SendStream};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::net::SocketAddr;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

/// A message sent by the client over the control stream.
#[derive(Debug, Serialize, Deserialize)]
enum ClientMessage {
    ConnectTo(ConnectTo),
    EnableTerminalEncryption(EnableTerminalEncryption),
}

/// Message sent by the client to indicate the destination server it wishes
/// to connect to.
#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectTo {
    /// Authentication key, required to prevent misuse of the gateway server.
    pub authentication_key: String,
    /// Destination server to proxy the connection to.
    pub destination_server: SocketAddr,
}

/// Message sent by the client to inform the gateway of the shared
/// encryption secret it has agreed on with the server.
///
/// This encryption is only used between the gateway and the destination
/// server (thus "terminal").
#[derive(Debug, Serialize, Deserialize)]
pub struct EnableTerminalEncryption {
    pub key: [u8; 16],
}

#[derive(Debug, Serialize, Deserialize)]
enum GatewayMessage {
    /// Sent when the gateway has completed the ConnectTo request.
    AcknowledgeConnectTo,
    /// Sent when the gateway has received the encryption secret
    /// and has now enabled encryption for all future packets.
    AcknowledgeEnableTerminalEncryption,
}

/// Used to send and receive `Message`s.
struct Codec {
    framed: Framed<IoDuplex<RecvStream, SendStream>, LengthDelimitedCodec>,
}

impl Codec {
    pub fn new(send_stream: SendStream, recv_stream: RecvStream) -> Self {
        Self {
            framed: Framed::new(
                IoDuplex::new(recv_stream, send_stream),
                LengthDelimitedCodec::new(),
            ),
        }
    }

    pub async fn send_message(&mut self, message: &impl Serialize) -> anyhow::Result<()> {
        let bytes = encode(message)?;
        self.framed.send(bytes.into()).await?;
        Ok(())
    }

    pub async fn recv_message<M: DeserializeOwned>(&mut self) -> anyhow::Result<M> {
        let bytes = self
            .framed
            .next()
            .await
            .context("control stream: end of stream")??;
        let message = decode(&bytes)?;
        Ok(message)
    }
}

/// Wrapper over the control stream on the client's side.
pub struct ClientSide {
    codec: Codec,
}

impl ClientSide {
    /// Opens the control stream on the given connection.
    /// This should be the first stream opened.
    pub async fn open(connection: &Connection) -> anyhow::Result<Self> {
        let (send_stream, recv_stream) = connection.open_bi().await?;
        Ok(Self {
            codec: Codec::new(send_stream, recv_stream),
        })
    }

    /// Sends a ConnectTo message to the gateway,
    /// then waits for acknowledgement.
    pub async fn connect_to(
        &mut self,
        destination_server: SocketAddr,
        authentication_key: &str,
    ) -> anyhow::Result<()> {
        self.codec
            .send_message(&ClientMessage::ConnectTo(ConnectTo {
                destination_server,
                authentication_key: authentication_key.to_owned(),
            }))
            .await?;
        self.wait_for_ack(|msg| matches!(msg, GatewayMessage::AcknowledgeConnectTo))
            .await?;
        Ok(())
    }

    pub async fn enable_terminal_encryption(&mut self, key: [u8; 16]) -> anyhow::Result<()> {
        self.codec
            .send_message(&ClientMessage::EnableTerminalEncryption(
                EnableTerminalEncryption { key },
            ))
            .await?;
        self.wait_for_ack(|msg| matches!(msg, GatewayMessage::AcknowledgeEnableTerminalEncryption))
            .await?;
        Ok(())
    }

    async fn wait_for_ack(
        &mut self,
        expected_message: impl FnOnce(&GatewayMessage) -> bool,
    ) -> anyhow::Result<()> {
        let message: GatewayMessage = self.codec.recv_message().await?;
        if expected_message(&message) {
            Ok(())
        } else {
            Err(anyhow!("wrong acknowledgement received from gateway"))
        }
    }
}

/// Wrapper over the control stream on the gateway's side.
pub struct GatewaySide {
    codec: Codec,
}

impl GatewaySide {
    /// Waits for the control stream to be opened by the client,
    /// then takes control of it.
    ///
    /// This should be the first time the connection is used (i.e.
    /// immediately after it is accepted)
    pub async fn accept(connection: &Connection) -> anyhow::Result<Self> {
        let (send_stream, recv_stream) = connection.accept_bi().await?;
        Ok(Self {
            codec: Codec::new(send_stream, recv_stream),
        })
    }

    /// Waits for a `ConnectTo` message.
    pub async fn wait_for_connect_to(&mut self) -> anyhow::Result<ConnectTo> {
        self.wait_for_message(|msg| match msg {
            ClientMessage::ConnectTo(m) => Some(m),
            _ => None,
        })
        .await
    }

    pub async fn acknowledge_connect_to(&mut self) -> anyhow::Result<()> {
        self.codec
            .send_message(&GatewayMessage::AcknowledgeConnectTo)
            .await
    }

    /// Waits for an encryption message.
    pub async fn wait_for_terminal_encryption(
        &mut self,
    ) -> anyhow::Result<EnableTerminalEncryption> {
        self.wait_for_message(|msg| match msg {
            ClientMessage::EnableTerminalEncryption(m) => Some(m),
            _ => None,
        })
        .await
    }

    pub async fn acknowledge_terminal_encryption(&mut self) -> anyhow::Result<()> {
        self.codec
            .send_message(&GatewayMessage::AcknowledgeEnableTerminalEncryption)
            .await
    }

    async fn wait_for_message<M>(
        &mut self,
        map_message: impl FnOnce(ClientMessage) -> Option<M>,
    ) -> anyhow::Result<M> {
        let message = self.codec.recv_message().await?;
        map_message(message).ok_or_else(|| anyhow!("unexpected message received on control stream"))
    }
}

fn encode<T: Serialize>(value: &T) -> anyhow::Result<Vec<u8>> {
    bincode::options()
        .serialize(value)
        .map_err(anyhow::Error::from)
}

fn decode<T: DeserializeOwned>(bytes: &[u8]) -> anyhow::Result<T> {
    bincode::options()
        .deserialize(bytes)
        .map_err(anyhow::Error::from)
}
