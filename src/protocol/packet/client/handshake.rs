use minecraft_quic_proxy_macros::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
#[encoding(discriminant = "varint")]
pub enum Packet {
    #[encoding(id = 0x00)]
    Handshake(Handshake),
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct Handshake {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
