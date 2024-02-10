use minecraft_quic_proxy_macros::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode, strum::AsRefStr)]
#[encoding(discriminant = "varint")]
pub enum Packet {
    #[encoding(id = 0x00)]
    StatusRequest(StatusRequest),
    #[encoding(id = 0x01)]
    PingRequest(PingRequest),
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct StatusRequest {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PingRequest {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
