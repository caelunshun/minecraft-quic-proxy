use minecraft_quic_proxy_macros::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode, strum::AsRefStr)]
#[encoding(discriminant = "varint")]
pub enum Packet {
    #[encoding(id = 0x00)]
    LoginStart(LoginStart),
    #[encoding(id = 0x01)]
    EncryptionResponse(EncryptionResponse),
    #[encoding(id = 0x02)]
    LoginPluginResponse(LoginPluginResponse),
    #[encoding(id = 0x03)]
    LoginAcknowledged(LoginAcknowledged),
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct LoginStart {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct EncryptionResponse {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct LoginPluginResponse {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct LoginAcknowledged {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
