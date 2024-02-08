//! Enumerates the possible packet types across versions.
//!
//! Full parsing of packets is _not_ implemented. Only the necessary
//! fields required for protocol interception & optimization are decoded.
//! The remainder of the data is decoded as a Vec<u8> containing the rest
//! of the packet's bytes. (This enables roundtrip encoding/decoding without
//! loss of information.)

use crate::protocol::{Decode, Encode};

pub mod client;
pub mod server;

/// Type encoding for a side (client or server).
pub trait Side {
    type SendPacket<State: ProtocolState>: Encode;
    type RecvPacket<State: ProtocolState>: Decode;
}

pub mod side {
    use super::*;

    pub struct Server;
    impl Side for Server {
        type SendPacket<State: ProtocolState> = State::ServerPacket;
        type RecvPacket<State: ProtocolState> = State::ClientPacket;
    }

    pub struct Client;
    impl Side for Client {
        type SendPacket<State: ProtocolState> = State::ClientPacket;
        type RecvPacket<State: ProtocolState> = State::ServerPacket;
    }
}

/// Type encoding for a protocol state.
pub trait ProtocolState {
    /// Packet type sent by the server in this state.
    type ServerPacket: Encode + Decode;
    /// Packet type sent by the client in this state.
    type ClientPacket: Encode + Decode;
}

pub mod state {
    use super::*;

    pub struct Handshake;
    impl ProtocolState for Handshake {
        type ServerPacket = ();
        type ClientPacket = client::handshake::Packet;
    }

    pub struct Status;
    impl ProtocolState for Status {
        type ServerPacket = server::status::Packet;
        type ClientPacket = client::status::Packet;
    }

    pub struct Login;
    impl ProtocolState for Login {
        type ServerPacket = server::login::Packet;
        type ClientPacket = client::login::Packet;
    }

    pub struct Configuration;
    impl ProtocolState for Configuration {
        type ServerPacket = server::configuration::Packet;
        type ClientPacket = client::configuration::Packet;
    }

    pub struct Play;
    impl ProtocolState for Play {
        type ServerPacket = server::play::Packet;
        type ClientPacket = client::play::Packet;
    }
}
