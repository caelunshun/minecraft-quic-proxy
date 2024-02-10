//! Implements the clientside translation layer
//! from TCP to QUIC.

use crate::{
    control_stream,
    protocol::packet::{client, client::handshake::NextState, side, state},
    proxy::{PacketIo, Proxy, QuicPacketIo, SingleQuicPacketIo, VanillaPacketIo},
};
use anyhow::Context;
use quinn::{Connection, Endpoint};
use std::{net::SocketAddr, ops::ControlFlow, thread};
use tokio::{
    net::{TcpListener, TcpStream},
    runtime,
    sync::oneshot,
    task::LocalSet,
};

pub struct ClientHandle {
    bound_port: u16,
    encryption_key_tx: Option<oneshot::Sender<[u8; 16]>>,
}

impl ClientHandle {
    /// Opens a new client.
    pub async fn open(
        endpoint: &Endpoint,
        gateway_host: &str,
        gateway_port: u16,
        destination_address: SocketAddr,
        authentication_key: &str,
    ) -> anyhow::Result<Self> {
        let client_listener = TcpListener::bind("127.0.0.1:0").await?;
        let bound_port = client_listener.local_addr()?.port();

        let gateway_address: SocketAddr = format!("{gateway_host}:{gateway_port}").parse()?;
        let gateway_connection = endpoint.connect(gateway_address, gateway_host)?.await?;

        let mut control_stream = control_stream::ClientSide::open(&gateway_connection).await?;
        control_stream
            .connect_to(destination_address, authentication_key)
            .await?;

        let (encryption_key_tx, encryption_key_rx) = oneshot::channel();

        let runtime = runtime::Handle::current();
        thread::spawn(move || {
            let local_set = LocalSet::new();
            local_set.spawn_local(async move {
                let client_stream = match client_listener.accept().await {
                    Ok((stream, _)) => stream,
                    Err(e) => {
                        tracing::warn!("Failed to accept connection from client: {e}");
                        return;
                    }
                };
                let client = match Client::new(
                    &gateway_connection,
                    client_stream,
                    control_stream,
                    encryption_key_rx,
                )
                .await
                {
                    Ok(client) => client,
                    Err(e) => {
                        tracing::warn!("Failed to initialize client: {e}");
                        return;
                    }
                };
                client.run().await;
            });

            runtime.block_on(local_set);
        });

        Ok(Self {
            encryption_key_tx: Some(encryption_key_tx),
            bound_port,
        })
    }

    /// Sets the encryption key. This must be called immediately
    /// after the client sends EncryptionResponse.
    ///
    /// # Panics
    /// Panics if called multiple times.
    pub fn set_encryption_key(&mut self, key: [u8; 16]) {
        self.encryption_key_tx
            .take()
            .expect("called ClientHandle::set_encryption_key twice")
            .send(key)
            .ok();
    }

    /// Gets the port the client side is bound to.
    /// The client should connect over TCP to this port
    /// to initiate the proxying.
    pub fn bound_port(&self) -> u16 {
        self.bound_port
    }
}

struct Client {
    state: State,
    control_stream: control_stream::ClientSide,
    encryption_key_future: Option<oneshot::Receiver<[u8; 16]>>,
}

impl Client {
    pub async fn new(
        gateway_connection: &Connection,
        client_stream: TcpStream,
        control_stream: control_stream::ClientSide,
        encryption_key_future: oneshot::Receiver<[u8; 16]>,
    ) -> anyhow::Result<Self> {
        let state = State::Handshake(HandshakeState::new(gateway_connection, client_stream).await?);

        Ok(Self {
            state,
            control_stream,
            encryption_key_future: Some(encryption_key_future),
        })
    }

    pub async fn run(self) {
        if let Err(e) = self.run_inner().await {
            tracing::warn!("Error in connection: {e}");
        }
    }

    async fn run_inner(mut self) -> anyhow::Result<()> {
        loop {
            let new_state = match self.state {
                State::Handshake(handshake) => handshake.proxy_until_next_state().await?,
                State::Status(status) => {
                    status.proxy().await?;
                    break;
                }
                State::Login(login) => {
                    login
                        .proxy_until_next_state(
                            &mut self.control_stream,
                            self.encryption_key_future
                                .take()
                                .expect("multiple login states?"),
                        )
                        .await?
                }
                State::Configuration(config) => config.proxy_until_next_state().await?,
                State::Play(play) => {
                    play.proxy().await?;
                    break;
                }
            };
            self.state = new_state;
        }
        Ok(())
    }
}

enum State {
    Handshake(HandshakeState),
    Status(StatusState),
    Login(LoginState),
    Configuration(ConfigurationState),
    Play(PlayState),
}

struct HandshakeState {
    gateway: SingleQuicPacketIo<side::Client, state::Handshake>,
    client: VanillaPacketIo<side::Server, state::Handshake>,
}

impl HandshakeState {
    pub async fn new(
        gateway_connection: &Connection,
        client_stream: TcpStream,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            gateway: SingleQuicPacketIo::new(gateway_connection).await?,
            client: VanillaPacketIo::new(client_stream)?,
        })
    }

    /// Proxies packets until we arrive at the next state, returning the new state.
    pub async fn proxy_until_next_state(self) -> anyhow::Result<State> {
        let client::handshake::Packet::Handshake(handshake) = self.client.recv_packet().await?;
        self.gateway
            .send_packet(client::handshake::Packet::Handshake(handshake.clone()))
            .await?;

        // HACK: "consume" the receive stream for the Handshake state now that the gateway
        // will close it. Otherwise, the receive stream for Handshake is incorrectly
        // used for the next state (since no data has been received on it and therefore
        // QUIC has not notified us of its existence).
        self.gateway.connection().accept_uni().await?;

        match handshake.next_state {
            NextState::Status => self.into_status().await.map(State::Status),
            NextState::Login => self.into_login().await.map(State::Login),
        }
    }

    pub async fn into_status(self) -> anyhow::Result<StatusState> {
        tracing::debug!("Transition to Status state");
        let gateway = self.gateway.switch_state().await?;
        let client = self.client.switch_state();
        Ok(StatusState { gateway, client })
    }

    pub async fn into_login(self) -> anyhow::Result<LoginState> {
        tracing::debug!("Transition to Login state");
        let gateway = self.gateway.switch_state().await?;
        let client = self.client.switch_state();
        Ok(LoginState { gateway, client })
    }
}

struct StatusState {
    gateway: SingleQuicPacketIo<side::Client, state::Status>,
    client: VanillaPacketIo<side::Server, state::Status>,
}

impl StatusState {
    pub async fn proxy(self) -> anyhow::Result<()> {
        Proxy::new(self.client, self.gateway)
            .run(
                |_| ControlFlow::Continue(()),
                |_| ControlFlow::<()>::Continue(()),
            )
            .await
    }
}

struct LoginState {
    gateway: SingleQuicPacketIo<side::Client, state::Login>,
    client: VanillaPacketIo<side::Server, state::Login>,
}

impl LoginState {
    pub async fn proxy_until_next_state(
        mut self,
        control_stream: &mut control_stream::ClientSide,
        encryption_key: oneshot::Receiver<[u8; 16]>,
    ) -> anyhow::Result<State> {
        let mut proxy = Proxy::new(self.client, self.gateway);
        let mut encryption_key = Some(encryption_key);

        #[derive(Debug)]
        enum Status {
            EnableEncryption,
            Finish,
        }

        loop {
            let status = proxy
                .run(
                    |client_packet| {
                        if let client::login::Packet::EncryptionResponse(_) = client_packet {
                            ControlFlow::Break(Status::EnableEncryption)
                        } else if let client::login::Packet::LoginAcknowledged(_) = client_packet {
                            ControlFlow::Break(Status::Finish)
                        } else {
                            ControlFlow::Continue(())
                        }
                    },
                    |_| ControlFlow::Continue(()),
                )
                .await?;

            tracing::debug!("Login loop status: {status:?}");

            match status {
                Status::EnableEncryption => {
                    let key = encryption_key
                        .take()
                        .context("multiple EncryptionResponse")?
                        .await?;
                    control_stream.enable_terminal_encryption(key).await?;
                }
                Status::Finish => break,
            }
        }

        let (client, gateway) = proxy.into_parts();
        self.gateway = gateway;
        self.client = client;

        self.into_configuration().await.map(State::Configuration)
    }

    pub async fn into_configuration(self) -> anyhow::Result<ConfigurationState> {
        tracing::debug!("Transition to Configuration state");
        let gateway = self.gateway.switch_state().await?;
        let client = self.client.switch_state();
        Ok(ConfigurationState { gateway, client })
    }
}

struct ConfigurationState {
    gateway: SingleQuicPacketIo<side::Client, state::Configuration>,
    client: VanillaPacketIo<side::Server, state::Configuration>,
}

impl ConfigurationState {
    pub async fn proxy_until_next_state(mut self) -> anyhow::Result<State> {
        let mut proxy = Proxy::new(self.client, self.gateway);

        proxy
            .run(
                |client_packet| {
                    if let client::configuration::Packet::FinishConfiguration(_) = client_packet {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                },
                |_| ControlFlow::Continue(()),
            )
            .await?;

        (self.client, self.gateway) = proxy.into_parts();
        self.into_play().await.map(State::Play)
    }

    pub async fn into_play(self) -> anyhow::Result<PlayState> {
        tracing::debug!("Transition to Play state");
        let gateway = QuicPacketIo::new(self.gateway.connection().clone()).await?;
        let client = self.client.switch_state();
        Ok(PlayState { gateway, client })
    }
}

struct PlayState {
    gateway: QuicPacketIo<side::Client>,
    client: VanillaPacketIo<side::Server, state::Play>,
}

impl PlayState {
    pub async fn proxy(self) -> anyhow::Result<()> {
        Proxy::new(self.client, self.gateway)
            .run(
                |_| ControlFlow::<()>::Continue(()),
                |_| ControlFlow::Continue(()),
            )
            .await?;
        Ok(())
    }
}
