use crate::{
    position::{BlockPosition, ChunkPosition},
    protocol::{decoder, Decode, Decoder, Encode, Encoder},
};
use minecraft_quic_proxy_macros::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode, strum::AsRefStr)]
#[encoding(discriminant = "varint")]
pub enum Packet {
    #[encoding(id = 0x00)]
    BundleDelimiter(BundleDelimiter),
    #[encoding(id = 0x01)]
    SpawnEntity(SpawnEntity),
    #[encoding(id = 0x02)]
    SpawnExperienceOrb(SpawnExperienceOrb),
    #[encoding(id = 0x03)]
    EntityAnimation(EntityAnimation),
    #[encoding(id = 0x04)]
    AwardStatistics(AwardStatistics),
    #[encoding(id = 0x05)]
    AcknowledgeBlockChange(AcknowledgeBlockChange),
    #[encoding(id = 0x06)]
    SetBlockDestroyStage(SetBlockDestroyStage),
    #[encoding(id = 0x07)]
    BlockEntityData(BlockEntityData),
    #[encoding(id = 0x08)]
    BlockAction(BlockAction),
    #[encoding(id = 0x09)]
    BlockUpdate(BlockUpdate),
    #[encoding(id = 0x0a)]
    BossBar(BossBar),
    #[encoding(id = 0x0b)]
    ChangeDifficulty(ChangeDifficulty),
    #[encoding(id = 0x0c)]
    ChunkBatchFinished(ChunkBatchFinished),
    #[encoding(id = 0x0d)]
    ChunkBatchStart(ChunkBatchStart),
    #[encoding(id = 0x0e)]
    ChunkBiomes(ChunkBiomes),
    #[encoding(id = 0x0f)]
    ClearTitles(ClearTitles),
    #[encoding(id = 0x10)]
    CommandSuggestions(CommandSuggestions),
    #[encoding(id = 0x11)]
    Commands(Commands),
    #[encoding(id = 0x12)]
    CloseContainer(CloseContainer),
    #[encoding(id = 0x13)]
    SetContainerContents(SetContainerContents),
    #[encoding(id = 0x14)]
    SetContainerProperty(SetContainerProperty),
    #[encoding(id = 0x15)]
    SetContainerSlot(SetContainerSlot),
    #[encoding(id = 0x16)]
    SetCooldown(SetCooldown),
    #[encoding(id = 0x17)]
    ChatSuggestions(ChatSuggestions),
    #[encoding(id = 0x18)]
    PluginMessage(PluginMessage),
    #[encoding(id = 0x19)]
    DamageEvent(DamageEvent),
    #[encoding(id = 0x1a)]
    DeleteMessage(DeleteMessage),
    #[encoding(id = 0x1b)]
    Disconnect(Disconnect),
    #[encoding(id = 0x1c)]
    DisguisedChatMessage(DisguisedChatMessage),
    #[encoding(id = 0x1d)]
    EntityEvent(EntityEvent),
    #[encoding(id = 0x1e)]
    Explosion(Explosion),
    #[encoding(id = 0x1f)]
    UnloadChunk(UnloadChunk),
    #[encoding(id = 0x20)]
    GameEvent(GameEvent),
    #[encoding(id = 0x21)]
    OpenHorseScreen(OpenHorseScreen),
    #[encoding(id = 0x22)]
    HurtAnimation(HurtAnimation),
    #[encoding(id = 0x23)]
    InitializeWorldBorder(InitializeWorldBorder),
    #[encoding(id = 0x24)]
    KeepAlive(KeepAlive),
    #[encoding(id = 0x25)]
    ChunkAndLightData(ChunkAndLightData),
    #[encoding(id = 0x26)]
    WorldEvent(WorldEvent),
    #[encoding(id = 0x27)]
    Particle(Particle),
    #[encoding(id = 0x28)]
    UpdateLight(UpdateLight),
    #[encoding(id = 0x29)]
    Login(Login),
    #[encoding(id = 0x2a)]
    MapData(MapData),
    #[encoding(id = 0x2b)]
    MerchantOffers(MerchantOffers),
    #[encoding(id = 0x2c)]
    UpdateEntityPosition(UpdateEntityPosition),
    #[encoding(id = 0x2d)]
    UpdateEntityPositionAndRotation(UpdateEntityPositionAndRotation),
    #[encoding(id = 0x2e)]
    UpdateEntityRotation(UpdateEntityRotation),
    #[encoding(id = 0x2f)]
    MoveVehicle(MoveVehicle),
    #[encoding(id = 0x30)]
    OpenBook(OpenBook),
    #[encoding(id = 0x31)]
    OpenScreen(OpenScreen),
    #[encoding(id = 0x32)]
    OpenSignEditor(OpenSignEditor),
    #[encoding(id = 0x33)]
    Ping(Ping),
    #[encoding(id = 0x34)]
    PingResponse(PingResponse),
    #[encoding(id = 0x35)]
    PlaceGhostRecipe(PlaceGhostRecipe),
    #[encoding(id = 0x36)]
    PlayerAbilities(PlayerAbilities),
    #[encoding(id = 0x37)]
    PlayerChatMessage(PlayerChatMessage),
    #[encoding(id = 0x38)]
    EndCombat(EndCombat),
    #[encoding(id = 0x39)]
    EnterCombat(EnterCombat),
    #[encoding(id = 0x3a)]
    CombatDeath(CombatDeath),
    #[encoding(id = 0x3b)]
    PlayerInfoRemove(PlayerInfoRemove),
    #[encoding(id = 0x3c)]
    PlayerInfoUpdate(PlayerInfoUpdate),
    #[encoding(id = 0x3d)]
    LookAt(LookAt),
    #[encoding(id = 0x3e)]
    SynchronizePlayerPosition(SynchronizePlayerPosition),
    #[encoding(id = 0x3f)]
    UpdateRecipeBook(UpdateRecipeBook),
    #[encoding(id = 0x40)]
    RemoveEntities(RemoveEntities),
    #[encoding(id = 0x41)]
    RemoveEntityEffect(RemoveEntityEffect),
    #[encoding(id = 0x42)]
    ResetScore(ResetScore),
    #[encoding(id = 0x43)]
    RemoveResourcePack(RemoveResourcePack),
    #[encoding(id = 0x44)]
    AddResourcePack(AddResourcePack),
    #[encoding(id = 0x45)]
    Respawn(Respawn),
    #[encoding(id = 0x46)]
    SetHeadRotation(SetHeadRotation),
    #[encoding(id = 0x47)]
    UpdateSectionBlocks(UpdateSectionBlocks),
    #[encoding(id = 0x48)]
    SelectAdvancementsTab(SelectAdvancementsTab),
    #[encoding(id = 0x49)]
    ServerData(ServerData),
    #[encoding(id = 0x4a)]
    SetActionBarText(SetActionBarText),
    #[encoding(id = 0x4b)]
    SetWorldBorderCenter(SetWorldBorderCenter),
    #[encoding(id = 0x4c)]
    SetWorldBorderLerpSize(SetWorldBorderLerpSize),
    #[encoding(id = 0x4d)]
    SetWorldBorderSize(SetWorldBorderSize),
    #[encoding(id = 0x4e)]
    SetWorldBorderWarningDelay(SetWorldBorderWarningDelay),
    #[encoding(id = 0x4f)]
    SetWorldBorderWarningDistance(SetWorldBorderWarningDistance),
    #[encoding(id = 0x50)]
    SetCamera(SetCamera),
    #[encoding(id = 0x51)]
    SetHeldItem(SetHeldItem),
    #[encoding(id = 0x52)]
    SetCenterChunk(SetCenterChunk),
    #[encoding(id = 0x53)]
    SetViewDistance(SetViewDistance),
    #[encoding(id = 0x54)]
    SetDefaultSpawnPosition(SetDefaultSpawnPosition),
    #[encoding(id = 0x55)]
    DisplayObjective(DisplayObjective),
    #[encoding(id = 0x56)]
    SetEntityMetadata(SetEntityMetadata),
    #[encoding(id = 0x57)]
    LinkEntities(LinkEntities),
    #[encoding(id = 0x58)]
    SetEntityVelocity(SetEntityVelocity),
    #[encoding(id = 0x59)]
    SetEquipment(SetEquipment),
    #[encoding(id = 0x5a)]
    SetExperience(SetExperience),
    #[encoding(id = 0x5b)]
    SetHealth(SetHealth),
    #[encoding(id = 0x5c)]
    UpdateObjectives(UpdateObjectives),
    #[encoding(id = 0x5d)]
    SetPassengers(SetPassengers),
    #[encoding(id = 0x5e)]
    UpdateTeams(UpdateTeams),
    #[encoding(id = 0x5f)]
    UpdateScore(UpdateScore),
    #[encoding(id = 0x60)]
    SetSimulationDistance(SetSimulationDistance),
    #[encoding(id = 0x61)]
    SetSubtitleText(SetSubtitleText),
    #[encoding(id = 0x62)]
    UpdateTime(UpdateTime),
    #[encoding(id = 0x63)]
    SetTitleText(SetTitleText),
    #[encoding(id = 0x64)]
    SetTitleAnimationTimes(SetTitleAnimationTimes),
    #[encoding(id = 0x65)]
    EntitySoundEffect(EntitySoundEffect),
    #[encoding(id = 0x66)]
    SoundEffect(SoundEffect),
    #[encoding(id = 0x67)]
    StartConfiguration(StartConfiguration),
    #[encoding(id = 0x68)]
    StopSound(StopSound),
    #[encoding(id = 0x69)]
    SystemChatMessage(SystemChatMessage),
    #[encoding(id = 0x6a)]
    SetTabListHeaderAndFooter(SetTabListHeaderAndFooter),
    #[encoding(id = 0x6b)]
    TagQueryResponse(TagQueryResponse),
    #[encoding(id = 0x6c)]
    PickUpItem(PickUpItem),
    #[encoding(id = 0x6d)]
    TeleportEntity(TeleportEntity),
    #[encoding(id = 0x6e)]
    SetTickingState(SetTickingState),
    #[encoding(id = 0x6f)]
    StepTick(StepTick),
    #[encoding(id = 0x70)]
    UpdateAdvancements(UpdateAdvancements),
    #[encoding(id = 0x71)]
    UpdateAttributes(UpdateAttributes),
    #[encoding(id = 0x72)]
    EntityEffect(EntityEffect),
    #[encoding(id = 0x73)]
    UpdateRecipes(UpdateRecipes),
    #[encoding(id = 0x74)]
    UpdateTags(UpdateTags),
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct BundleDelimiter {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SpawnEntity {
    #[encoding(varint)]
    pub entity_id: i32,
    pub uuid: u128,
    #[encoding(varint)]
    pub kind: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    #[encoding(angle)]
    pub pitch: f32,
    #[encoding(angle)]
    pub yaw: f32,
    #[encoding(angle)]
    pub head_yaw: f32,
    #[encoding(varint)]
    pub data: i32,
    pub velocity_x: i16,
    pub velocity_y: i16,
    pub velocity_z: i16,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SpawnExperienceOrb {
    #[encoding(varint)]
    pub entity_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub amount: u16,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct EntityAnimation {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct AwardStatistics {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct AcknowledgeBlockChange {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetBlockDestroyStage {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct BlockEntityData {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct BlockAction {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct BlockUpdate {
    pub position: BlockPosition,
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct BossBar {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct ChangeDifficulty {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct ChunkBatchFinished {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct ChunkBatchStart {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct ChunkBiomes {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct ClearTitles {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct CommandSuggestions {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct Commands {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct CloseContainer {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetContainerContents {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetContainerProperty {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetContainerSlot {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetCooldown {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct ChatSuggestions {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct PluginMessage {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct DamageEvent {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct DeleteMessage {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct Disconnect {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct DisguisedChatMessage {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct EntityEvent {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct Explosion {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct UnloadChunk {
    pub chunk_z: i32,
    pub chunk_x: i32,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct GameEvent {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct OpenHorseScreen {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct HurtAnimation {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct InitializeWorldBorder {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct KeepAlive {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct ChunkAndLightData {
    pub chunk_x: i32,
    pub chunk_z: i32,
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct WorldEvent {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct Particle {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateLight {
    #[encoding(varint)]
    pub chunk_x: i32,
    #[encoding(varint)]
    pub chunk_z: i32,
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct Login {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct MapData {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct MerchantOffers {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateEntityPosition {
    #[encoding(varint)]
    pub entity_id: i32,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateEntityPositionAndRotation {
    #[encoding(varint)]
    pub entity_id: i32,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    #[encoding(angle)]
    pub yaw: f32,
    #[encoding(angle)]
    pub pitch: f32,
    pub on_ground: bool,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateEntityRotation {
    #[encoding(varint)]
    pub entity_id: i32,
    #[encoding(angle)]
    pub yaw: f32,
    #[encoding(angle)]
    pub pitch: f32,
    pub on_ground: bool,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct MoveVehicle {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct OpenBook {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct OpenScreen {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct OpenSignEditor {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct Ping {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct PingResponse {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct PlaceGhostRecipe {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct PlayerAbilities {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct PlayerChatMessage {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct EndCombat {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct EnterCombat {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct CombatDeath {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct PlayerInfoRemove {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct PlayerInfoUpdate {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct LookAt {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SynchronizePlayerPosition {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateRecipeBook {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone)]
pub struct RemoveEntities {
    pub entities: Vec<i32>,
}

impl Encode for RemoveEntities {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_var_int(self.entities.len().try_into().unwrap_or(i32::MAX));
        for id in &self.entities {
            encoder.write_var_int(*id);
        }
    }
}
impl Decode for RemoveEntities {
    fn decode(decoder: &mut Decoder) -> decoder::Result<Self> {
        let length = decoder.read_var_int()?;
        let mut entities = Vec::new();
        for _ in 0..length {
            entities.push(decoder.read_var_int()?);
        }
        Ok(Self { entities })
    }
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct RemoveEntityEffect {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct ResetScore {
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
pub struct Respawn {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetHeadRotation {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateSectionBlocks {
    pub chunk_section_position: i64,
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}

impl UpdateSectionBlocks {
    pub fn chunk_position(&self) -> ChunkPosition {
        ChunkPosition {
            x: (self.chunk_section_position >> 42) as i32,
            z: (self.chunk_section_position << 22 >> 42) as i32,
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SelectAdvancementsTab {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct ServerData {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetActionBarText {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetWorldBorderCenter {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetWorldBorderLerpSize {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetWorldBorderSize {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetWorldBorderWarningDelay {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetWorldBorderWarningDistance {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetCamera {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetHeldItem {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetCenterChunk {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetViewDistance {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetDefaultSpawnPosition {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct DisplayObjective {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetEntityMetadata {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct LinkEntities {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetEntityVelocity {
    #[encoding(varint)]
    pub entity_id: i32,
    pub velocity_x: i16,
    pub velocity_y: i16,
    pub velocity_z: i16,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetEquipment {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetExperience {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetHealth {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateObjectives {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetPassengers {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateTeams {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateScore {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetSimulationDistance {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetSubtitleText {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateTime {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetTitleText {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetTitleAnimationTimes {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct EntitySoundEffect {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SoundEffect {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct StartConfiguration {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct StopSound {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SystemChatMessage {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetTabListHeaderAndFooter {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct TagQueryResponse {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct PickUpItem {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct TeleportEntity {
    #[encoding(varint)]
    pub entity_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    #[encoding(angle)]
    pub yaw: f32,
    #[encoding(angle)]
    pub pitch: f32,
    pub on_ground: bool,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct SetTickingState {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct StepTick {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateAdvancements {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateAttributes {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct EntityEffect {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateRecipes {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
#[derive(Debug, Clone, Encode, Decode)]
pub struct UpdateTags {
    #[encoding(length_prefix = "inferred")]
    pub ignored_data: Vec<u8>,
}
