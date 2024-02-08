use minecraft_quic_proxy_macros::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
#[encoding(discriminant = "varint")]
pub enum Packet {
    #[encoding(id = 0x00)]
    PluginMessage(PluginMessage),
    #[encoding(id = 0x01)]
    Disconnect(Disconnect),
    #[encoding(id = 0x02)]
    FinishConfiguration(FinishConfiguration),
    #[encoding(id = 0x03)]
    KeepAlive(KeepAlive),
    #[encoding(id = 0x04)]
    Ping(Ping),
    #[encoding(id = 0x05)]
    RegistryData(RegistryData),
    #[encoding(id = 0x06)]
    RemoveResourcePack(RemoveResourcePack),
    #[encoding(id = 0x07)]
    AddResourcePack(AddResourcePack),
    #[encoding(id = 0x08)]
    FeatureFlags(FeatureFlags),
    #[encoding(id = 0x09)]
    UpdateTags(UpdateTags),
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PluginMessage {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct Disconnect {
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
pub struct Ping {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct RegistryData {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct RemoveResourcePack {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct AddResourcePack {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct FeatureFlags {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateTags {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
