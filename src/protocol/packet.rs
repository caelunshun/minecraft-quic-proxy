//! Enumerates the possible packet types across versions.
//!
//! Full parsing of packets is _not_ implemented. Only the necessary
//! fields required for protocol interception & optimization are decoded.
//! The remainder of the data is decoded as a Vec<u8> containing the rest
//! of the packet's bytes. (This enables roundtrip encoding/decoding without
//! loss of information.)

use crate::protocol::{Decode, Encode};
use std::fmt::Debug;

pub mod client;
pub mod server;

/// Type encoding for a side (client or server).
pub trait Side: Send + Sync + 'static + Copy + Clone {
    type SendPacket<State: ProtocolState>: Encode + Debug + AsRef<str> + Send + 'static;
    type RecvPacket<State: ProtocolState>: Decode + Debug + AsRef<str> + Send + 'static;
}

pub mod side {
    use super::*;

    #[derive(Debug, Copy, Clone)]
    pub struct Server;
    impl Side for Server {
        type SendPacket<State: ProtocolState> = State::ServerPacket;
        type RecvPacket<State: ProtocolState> = State::ClientPacket;
    }

    #[derive(Debug, Copy, Clone)]
    pub struct Client;
    impl Side for Client {
        type SendPacket<State: ProtocolState> = State::ClientPacket;
        type RecvPacket<State: ProtocolState> = State::ServerPacket;
    }
}

/// Type encoding for a protocol state.
pub trait ProtocolState: Send + Sync + 'static {
    /// Packet type sent by the server in this state.
    type ServerPacket: Encode + Decode + Debug + AsRef<str> + Send + 'static;
    /// Packet type sent by the client in this state.
    type ClientPacket: Encode + Decode + Debug + AsRef<str> + Send + 'static;
}

pub mod state {
    use super::*;
    use minecraft_quic_proxy_macros::{Decode, Encode};

    #[derive(Debug, Copy, Clone)]
    pub struct Handshake;
    impl ProtocolState for Handshake {
        type ServerPacket = EmptyPacket;
        type ClientPacket = client::handshake::Packet;
    }

    #[derive(Encode, Decode, Debug, Clone)]
    pub struct EmptyPacket;

    impl AsRef<str> for EmptyPacket {
        fn as_ref(&self) -> &str {
            ""
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct Status;
    impl ProtocolState for Status {
        type ServerPacket = server::status::Packet;
        type ClientPacket = client::status::Packet;
    }

    #[derive(Debug, Copy, Clone)]
    pub struct Login;
    impl ProtocolState for Login {
        type ServerPacket = server::login::Packet;
        type ClientPacket = client::login::Packet;
    }

    #[derive(Debug, Copy, Clone)]
    pub struct Configuration;
    impl ProtocolState for Configuration {
        type ServerPacket = server::configuration::Packet;
        type ClientPacket = client::configuration::Packet;
    }

    #[derive(Debug, Copy, Clone)]
    pub struct Play;
    impl ProtocolState for Play {
        type ServerPacket = server::play::Packet;
        type ClientPacket = client::play::Packet;
    }
}
