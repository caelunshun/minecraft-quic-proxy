use minecraft_quic_proxy_macros::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
#[encoding(discriminant = "varint")]
pub enum Packet {
    #[encoding(id = 0x00)]
    ClientInformation(ClientInformation),
    #[encoding(id = 0x01)]
    PluginMessage(PluginMessage),
    #[encoding(id = 0x02)]
    FinishConfiguration(FinishConfiguration),
    #[encoding(id = 0x03)]
    KeepAlive(KeepAlive),
    #[encoding(id = 0x04)]
    Pong(Pong),
    #[encoding(id = 0x05)]
    ResourcePackResponse(ResourcePackResponse),
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ClientInformation {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PluginMessage {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct FinishConfiguration {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct KeepAlive {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct Pong {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ResourcePackResponse {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
