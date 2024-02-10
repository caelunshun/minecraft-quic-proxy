use minecraft_quic_proxy_macros::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode, strum::AsRefStr)]
#[encoding(discriminant = "varint")]
pub enum Packet {
    #[encoding(id = 0x00)]
    ConfirmTeleportation(ConfirmTeleportation),
    #[encoding(id = 0x01)]
    QueryBlockEntityTag(QueryBlockEntityTag),
    #[encoding(id = 0x02)]
    ChangeDifficulty(ChangeDifficulty),
    #[encoding(id = 0x03)]
    AcknowledgeMessage(AcknowledgeMessage),
    #[encoding(id = 0x04)]
    ChatCommand(ChatCommand),
    #[encoding(id = 0x05)]
    ChatMessage(ChatMessage),
    #[encoding(id = 0x06)]
    PlayerSession(PlayerSession),
    #[encoding(id = 0x07)]
    ChunkBatchReceived(ChunkBatchReceived),
    #[encoding(id = 0x08)]
    ClientStatus(ClientStatus),
    #[encoding(id = 0x09)]
    ClientInformation(ClientInformation),
    #[encoding(id = 0x0a)]
    RequestCommandSuggestions(RequestCommandSuggestions),
    #[encoding(id = 0x0b)]
    AcknowledgeConfiguration(AcknowledgeConfiguration),
    #[encoding(id = 0x0c)]
    ClickContainerButton(ClickContainerButton),
    #[encoding(id = 0x0d)]
    ClickContainer(ClickContainer),
    #[encoding(id = 0x0e)]
    CloseContainer(CloseContainer),
    #[encoding(id = 0x0f)]
    ChangeContainerSlotState(ChangeContainerSlotState),
    #[encoding(id = 0x10)]
    PluginMessage(PluginMessage),
    #[encoding(id = 0x11)]
    EditBook(EditBook),
    #[encoding(id = 0x12)]
    QueryEntityTag(QueryEntityTag),
    #[encoding(id = 0x13)]
    Interact(Interact),
    #[encoding(id = 0x14)]
    JigsawGenerate(JigsawGenerate),
    #[encoding(id = 0x15)]
    KeepAlive(KeepAlive),
    #[encoding(id = 0x16)]
    LockDifficulty(LockDifficulty),
    #[encoding(id = 0x17)]
    SetPlayerPosition(SetPlayerPosition),
    #[encoding(id = 0x18)]
    SetPlayerPositionAndRotation(SetPlayerPositionAndRotation),
    #[encoding(id = 0x19)]
    SetPlayerRotation(SetPlayerRotation),
    #[encoding(id = 0x1a)]
    SetPlayerOnGround(SetPlayerOnGround),
    #[encoding(id = 0x1b)]
    MoveVehicle(MoveVehicle),
    #[encoding(id = 0x1c)]
    PaddleBoat(PaddleBoat),
    #[encoding(id = 0x1d)]
    PickItem(PickItem),
    #[encoding(id = 0x1e)]
    PingRequest(PingRequest),
    #[encoding(id = 0x1f)]
    PlaceRecipe(PlaceRecipe),
    #[encoding(id = 0x20)]
    PlayerAbilityState(PlayerAbilityState),
    #[encoding(id = 0x21)]
    PlayerAction(PlayerAction),
    #[encoding(id = 0x22)]
    PlayerCommand(PlayerCommand),
    #[encoding(id = 0x23)]
    PlayerInput(PlayerInput),
    #[encoding(id = 0x24)]
    Pong(Pong),
    #[encoding(id = 0x25)]
    ChangeRecipeBookSettings(ChangeRecipeBookSettings),
    #[encoding(id = 0x26)]
    SetSeenRecipe(SetSeenRecipe),
    #[encoding(id = 0x27)]
    RenameItem(RenameItem),
    #[encoding(id = 0x28)]
    ResourcePackResponse(ResourcePackResponse),
    #[encoding(id = 0x29)]
    SeenAdvancements(SeenAdvancements),
    #[encoding(id = 0x2a)]
    SelectTrade(SelectTrade),
    #[encoding(id = 0x2b)]
    SetBeaconEffect(SetBeaconEffect),
    #[encoding(id = 0x2c)]
    SetHeldItem(SetHeldItem),
    #[encoding(id = 0x2d)]
    ProgramCommandBlock(ProgramCommandBlock),
    #[encoding(id = 0x2e)]
    ProgramCommandBlockMinecart(ProgramCommandBlockMinecart),
    #[encoding(id = 0x2f)]
    SetCreativeModeSlot(SetCreativeModeSlot),
    #[encoding(id = 0x30)]
    ProgramJigsawBlock(ProgramJigsawBlock),
    #[encoding(id = 0x31)]
    ProgramStructureBlock(ProgramStructureBlock),
    #[encoding(id = 0x32)]
    UpdateSign(UpdateSign),
    #[encoding(id = 0x33)]
    SwingArm(SwingArm),
    #[encoding(id = 0x34)]
    SpectatorTeleportToEntity(SpectatorTeleportToEntity),
    #[encoding(id = 0x35)]
    UseItemOn(UseItemOn),
    #[encoding(id = 0x36)]
    UseItem(UseItem),
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ConfirmTeleportation {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct QueryBlockEntityTag {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ChangeDifficulty {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct AcknowledgeMessage {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ChatCommand {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ChatMessage {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PlayerSession {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ChunkBatchReceived {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ClientStatus {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ClientInformation {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct RequestCommandSuggestions {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct AcknowledgeConfiguration {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ClickContainerButton {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ClickContainer {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CloseContainer {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ChangeContainerSlotState {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PluginMessage {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct EditBook {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct QueryEntityTag {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct Interact {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct JigsawGenerate {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct KeepAlive {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct LockDifficulty {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SetPlayerPosition {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SetPlayerPositionAndRotation {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SetPlayerRotation {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SetPlayerOnGround {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct MoveVehicle {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PaddleBoat {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PickItem {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PingRequest {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PlaceRecipe {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PlayerAbilityState {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PlayerAction {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PlayerCommand {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PlayerInput {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct Pong {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ChangeRecipeBookSettings {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SetSeenRecipe {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct RenameItem {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ResourcePackResponse {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SeenAdvancements {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SelectTrade {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SetBeaconEffect {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SetHeldItem {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ProgramCommandBlock {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ProgramCommandBlockMinecart {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SetCreativeModeSlot {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ProgramJigsawBlock {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ProgramStructureBlock {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateSign {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SwingArm {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SpectatorTeleportToEntity {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct UseItemOn {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct UseItem {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
