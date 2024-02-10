use minecraft_quic_proxy_macros::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode, strum::AsRefStr)]
#[encoding(discriminant = "varint")]
pub enum Packet {
    #[encoding(id = 0x00)]
    Handshake(Handshake),
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct Handshake {
    #[encoding(varint)]
    pub protocol_version: u32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: NextState,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Encode, Decode)]
#[encoding(discriminant = "varint")]
pub enum NextState {
    #[encoding(id = 1)]
    Status,
    #[encoding(id = 2)]
    Login,
}
