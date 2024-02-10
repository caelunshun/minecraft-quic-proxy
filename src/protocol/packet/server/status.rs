use minecraft_quic_proxy_macros::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode, strum::AsRefStr)]
#[encoding(discriminant = "varint")]
pub enum Packet {
    #[encoding(id = 0x00)]
    StatusResponse(StatusResponse),
    #[encoding(id = 0x01)]
    PingResponse(PingResponse),
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct StatusResponse {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PingResponse {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
