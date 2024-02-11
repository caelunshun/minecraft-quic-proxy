//! Implements the gateway server. This translates
//! from QUIC packets from the client to TCP sent to the destination server.

use crate::{
    control_stream,
    control_stream::EnableTerminalEncryption,
    protocol::{
        packet::{client, client::handshake::NextState, server, side, state},
        vanilla_codec::{CompressionThreshold, EncryptionKey},
    },
    proxy::{PacketIo, Proxy, QuicPacketIo, SingleQuicPacketIo, VanillaPacketIo},
    stream,
};
use anyhow::{anyhow, bail, Context};
use argon2::{PasswordHash, PasswordVerifier};
use quinn::{Connection, Endpoint};
use std::{ops::ControlFlow, thread, time::Duration};
use tokio::{net::TcpStream, runtime, task::LocalSet, time::timeout};

#[derive(Debug, Clone)]
pub enum AuthenticationKey {
    Plaintext(String),
    Hashed(String),
}

impl AuthenticationKey {
    pub fn is_correct(&self, key: &str) -> anyhow::Result<bool> {
        match self {
            Self::Plaintext(s) => Ok(s == key),
            Self::Hashed(s) => Ok(argon2::Argon2::default()
                .verify_password(
                    key.as_bytes(),
                    &PasswordHash::new(s).map_err(|_| {
                        anyhow!("configured authentication key is invalid Argon2 hash")
                    })?,
                )
                .is_ok()),
        }
    }
}

/// Runs a gateway server on the given endpoint.
pub async fn run(
    endpoint: &Endpoint,
    authentication_key: &AuthenticationKey,
) -> anyhow::Result<()> {
    loop {
        let connection = match endpoint.accept().await.context("endpoint closed")?.await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::warn!("Failed to accept connection: {e}");
                continue;
            }
        };

        tracing::info!("Accepted connection from {}", connection.remote_address());
        let authentication_key = authentication_key.clone();
        let runtime = runtime::Handle::current();
        thread::spawn(move || {
            let local_set = LocalSet::new();
            local_set.spawn_local(async move {
                if let Err(e) = drive_connection(connection, &authentication_key).await {
                    tracing::info!("Connection lost: {e:?}");
                }
            });
            runtime.block_on(local_set);
        });
    }
}

const CONFIGURATION_TIMEOUT: Duration = Duration::from_secs(30);

/// Accepts a new connection from a client.
async fn drive_connection(
    connection: Connection,
    authentication_key: &AuthenticationKey,
) -> anyhow::Result<()> {
    let mut control_stream = control_stream::GatewaySide::accept(&connection).await?;
    let connect_to = timeout(CONFIGURATION_TIMEOUT, control_stream.wait_for_connect_to()).await??;

    if !authentication_key.is_correct(&connect_to.authentication_key)? {
        bail!("client failed to present correct authentication key");
    }

    tracing::info!(
        "Connecting to destination server {}",
        connect_to.destination_server
    );
    let server_connection = TcpStream::connect(connect_to.destination_server).await?;
    tracing::info!(
        "Connected to destination server {}",
        connect_to.destination_server
    );
    let server_connection: VanillaPacketIo<side::Client, state::Handshake> =
        VanillaPacketIo::new(server_connection)?;
    control_stream.acknowledge_connect_to().await?;

    let client_connection: SingleQuicPacketIo<side::Server, state::Handshake> =
        SingleQuicPacketIo::new(&connection).await?;

    let (mut client_connection, mut server_connection) = match timeout(
        CONFIGURATION_TIMEOUT,
        configure_connection(server_connection, client_connection, &mut control_stream),
    )
    .await??
    {
        Some(conns) => conns,
        None => return Ok(()),
    };

    loop {
        let mut proxy = Proxy::new(client_connection, server_connection);
        proxy
            .run(
                |client_packet| {
                    if let client::play::Packet::AcknowledgeConfiguration(_) = client_packet {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                },
                |_| ControlFlow::<()>::Continue(()),
            )
            .await?;

        (client_connection, server_connection) = proxy.into_parts();
        control_stream
            .acknowledge_transition_play_to_config()
            .await?;
        tracing::debug!("Acknowledged transition to Configuration state");
        let (send, recv) = stream::open_bi(client_connection.connection(), "configuration").await?;
        let config_client_connection =
            SingleQuicPacketIo::from_streams(client_connection.connection(), send, recv);
        let config_server_connection = server_connection.switch_state();
        (client_connection, server_connection) =
            do_configuration(config_client_connection, config_server_connection).await?;
    }
}

type PlayConnections = (
    QuicPacketIo<side::Server>,
    VanillaPacketIo<side::Client, state::Play>,
);

/// Performs handling for a connection until it arrives in the Play state.
/// Returns `None` if the connection was a status connection and is therefore
/// now terminated.
async fn configure_connection(
    server_connection: VanillaPacketIo<side::Client, state::Handshake>,
    client_connection: SingleQuicPacketIo<side::Server, state::Handshake>,
    control_stream: &mut control_stream::GatewaySide,
) -> anyhow::Result<Option<PlayConnections>> {
    let client::handshake::Packet::Handshake(handshake) = client_connection.recv_packet().await?;
    server_connection
        .send_packet(client::handshake::Packet::Handshake(handshake.clone()))
        .await?;

    match handshake.next_state {
        NextState::Status => {
            tracing::debug!("Transition to Status state");
            handle_status(
                server_connection.switch_state(),
                client_connection.switch_state().await?,
            )
            .await?;
            Ok(None)
        }
        NextState::Login => {
            tracing::debug!("Transition to Login state");
            let (client_connection, server_connection) = (
                client_connection.switch_state::<state::Login>().await?,
                server_connection.switch_state::<state::Login>(),
            );

            #[derive(Debug)]
            enum Status {
                EnableEncryption,
                EnableCompression(CompressionThreshold),
                FinishLogin,
            }

            let mut proxy = Proxy::new(client_connection, server_connection);
            loop {
                let status = proxy
                    .run(
                        |client_packet| {
                            if let client::login::Packet::LoginAcknowledged(_) = client_packet {
                                ControlFlow::Break(Status::FinishLogin)
                            } else if let client::login::Packet::EncryptionResponse(_) =
                                client_packet
                            {
                                ControlFlow::Break(Status::EnableEncryption)
                            } else {
                                ControlFlow::Continue(())
                            }
                        },
                        |server_packet| {
                            if let server::login::Packet::SetCompression(packet) = server_packet {
                                if let Ok(threshold) = usize::try_from(packet.threshold) {
                                    return ControlFlow::Break(Status::EnableCompression(
                                        CompressionThreshold::new(threshold),
                                    ));
                                }
                            }
                            ControlFlow::Continue(())
                        },
                    )
                    .await?;
                tracing::debug!("Login loop status: {status:?}");

                match status {
                    Status::EnableEncryption => {
                        let EnableTerminalEncryption { key } =
                            control_stream.wait_for_terminal_encryption().await?;
                        proxy
                            .server_mut()
                            .enable_encryption(EncryptionKey::new(key));
                        control_stream.acknowledge_terminal_encryption().await?;
                    }
                    Status::EnableCompression(threshold) => {
                        proxy.server_mut().enable_compression(threshold);
                    }
                    Status::FinishLogin => break,
                }
            }

            let (client_connection, server_connection) = proxy.into_parts();
            do_configuration(
                client_connection.switch_state().await?,
                server_connection.switch_state(),
            )
            .await
            .map(Some)
        }
    }
}

async fn do_configuration(
    client_connection: SingleQuicPacketIo<side::Server, state::Configuration>,
    server_connection: VanillaPacketIo<side::Client, state::Configuration>,
) -> anyhow::Result<PlayConnections> {
    tracing::debug!("Transition to Configuration state");
    let mut proxy = Proxy::new(client_connection, server_connection);

    proxy
        .run(
            |packet| {
                if let client::configuration::Packet::FinishConfiguration(_) = packet {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
            |_| ControlFlow::Continue(()),
        )
        .await?;

    let (client_connection, server_connection) = proxy.into_parts();

    let new_client_connection =
        QuicPacketIo::<side::Server>::new(client_connection.connection().clone()).await?;

    tracing::debug!("Transition to Play state");
    Ok((new_client_connection, server_connection.switch_state()))
}

async fn handle_status(
    server_connection: VanillaPacketIo<side::Client, state::Status>,
    client_connection: SingleQuicPacketIo<side::Server, state::Status>,
) -> anyhow::Result<()> {
    Proxy::new(client_connection, server_connection)
        .run(
            |_| ControlFlow::<()>::Continue(()),
            |_| ControlFlow::Continue(()),
        )
        .await
        .ok();
    Ok(())
}
