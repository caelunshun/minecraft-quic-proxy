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
};
use anyhow::{bail, Context};
use quinn::{Connection, Endpoint};
use std::{ops::ControlFlow, thread, time::Duration};
use tokio::{net::TcpStream, runtime, task::LocalSet, time::timeout};

/// Runs a gateway server on the given endpoint.
pub async fn run(endpoint: &Endpoint, authentication_key: &str) -> anyhow::Result<()> {
    loop {
        let connection = endpoint.accept().await.context("endpoint closed")?.await?;

        tracing::info!("Accepted connection from {}", connection.remote_address());
        let authentication_key = authentication_key.to_owned();
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
async fn drive_connection(connection: Connection, authentication_key: &str) -> anyhow::Result<()> {
    let mut control_stream = control_stream::GatewaySide::accept(&connection).await?;
    let connect_to = timeout(CONFIGURATION_TIMEOUT, control_stream.wait_for_connect_to()).await??;

    if connect_to.authentication_key != authentication_key {
        bail!("client failed to present correct authentication key");
    }

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

    let (client_connection, server_connection) = match timeout(
        CONFIGURATION_TIMEOUT,
        configure_connection(server_connection, client_connection, &mut control_stream),
    )
    .await??
    {
        Some(conns) => conns,
        None => return Ok(()),
    };

    Proxy::new(client_connection, server_connection)
        .run(
            |_| ControlFlow::Continue(()),
            |_| ControlFlow::<()>::Continue(()),
        )
        .await?;

    Ok(())
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
