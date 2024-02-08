use minecraft_quic_proxy_macros::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
#[encoding(discriminant = "varint")]
pub enum Packet {
    #[encoding(id = 0x00)]
    Disconnect(Disconnect),
    #[encoding(id = 0x01)]
    EncryptionRequest(EncryptionRequest),
    #[encoding(id = 0x02)]
    LoginSuccess(LoginSuccess),
    #[encoding(id = 0x03)]
    SetCompression(SetCompression),
    #[encoding(id = 0x04)]
    LoginPluginRequest(LoginPluginRequest),
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct Disconnect {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct EncryptionRequest {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct LoginSuccess {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SetCompression {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct LoginPluginRequest {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
