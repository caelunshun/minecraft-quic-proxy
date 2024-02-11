use crate::{
    entity_id::EntityId,
    position::{EntityPosition, EntityPositionDelta},
    protocol::{
        packet,
        packet::{
            server,
            server::play::{
                TeleportEntity, UpdateEntityPosition, UpdateEntityPositionAndRotation,
                UpdateEntityRotation,
            },
            side, state,
            state::Play,
            Side,
        },
    },
};
use ahash::AHashMap;

/// Certain packets need to be modified to work correctly with
/// the QUIC protocol. For example, since entity movement packets
/// are sent unordered and unreliably, we need to translate all
/// relative movement packets to absolute ones.
///
/// This struct stores the necessary state to accomplish the above.
pub struct PacketTranslator {
    /// Last received position of each entity from the server.
    entity_positions: AHashMap<EntityId, EntityPosition>,
}

impl PacketTranslator {
    pub fn new() -> Self {
        Self {
            entity_positions: AHashMap::new(),
        }
    }

    fn register_entity_position(
        &mut self,
        entity_id: EntityId,
        position: impl Into<EntityPosition>,
    ) {
        self.entity_positions.insert(entity_id, position.into());
    }

    fn entity_position(&self, entity_id: EntityId) -> Option<EntityPosition> {
        let opt = self.entity_positions.get(&entity_id).copied();
        if opt.is_none() {
            tracing::warn!("Requesting position of entity {entity_id:?}, but it is not known.");
        }
        opt
    }

    fn unload_entity(&mut self, entity_id: EntityId) {
        self.entity_positions.remove(&entity_id);
    }

    fn clear_entities(&mut self) {
        self.entity_positions.clear();
    }
}

/// Trait implemented by `PacketTranslator` for sides Client and Server.
pub trait TranslatePacket<Side: packet::Side> {
    /// Translates a packet if needed.
    /// Returns the translated packet or `None`
    /// if no translation is required.
    fn translate_packet(
        &mut self,
        packet: &Side::SendPacket<state::Play>,
    ) -> Option<Side::SendPacket<state::Play>>;
}

impl TranslatePacket<side::Client> for PacketTranslator {
    fn translate_packet(
        &mut self,
        _packet: &<side::Client as Side>::SendPacket<Play>,
    ) -> Option<<side::Client as Side>::SendPacket<Play>> {
        // No translations currently needed for client=>server packets.
        None
    }
}

impl TranslatePacket<side::Server> for PacketTranslator {
    fn translate_packet(&mut self, packet: &server::play::Packet) -> Option<server::play::Packet> {
        use server::play::Packet;

        if let Packet::UpdateEntityPositionAndRotation(UpdateEntityPositionAndRotation {
            entity_id,
            pitch,
            yaw,
            ..
        })
        | Packet::UpdateEntityRotation(UpdateEntityRotation {
            entity_id,
            pitch,
            yaw,
            ..
        }) = packet
        {
            let entity_id = EntityId::new(*entity_id);
            if let Some(old_pos) = self.entity_position(entity_id) {
                // Note: position update handled in the match statement below.
                // We're just updating rotation here.
                self.register_entity_position(
                    entity_id,
                    EntityPosition {
                        pitch: *pitch,
                        yaw: *yaw,
                        ..old_pos
                    },
                );
            }
        }

        match packet {
            Packet::SpawnEntity(packet) => {
                self.register_entity_position(
                    EntityId::new(packet.entity_id),
                    (packet.x, packet.y, packet.z, packet.pitch, packet.yaw),
                );
                None
            }
            Packet::SpawnExperienceOrb(packet) => {
                self.register_entity_position(
                    EntityId::new(packet.entity_id),
                    (packet.x, packet.y, packet.z, 0.0, 0.0),
                );
                None
            }
            Packet::TeleportEntity(packet) => {
                self.register_entity_position(
                    EntityId::new(packet.entity_id),
                    (packet.x, packet.y, packet.z, packet.pitch, packet.yaw),
                );
                None
            }
            Packet::UpdateEntityPosition(UpdateEntityPosition {
                entity_id,
                delta_x,
                delta_y,
                delta_z,
                on_ground,
            })
            | Packet::UpdateEntityPositionAndRotation(UpdateEntityPositionAndRotation {
                entity_id,
                delta_x,
                delta_y,
                delta_z,
                on_ground,
                ..
            }) => {
                let entity_id = EntityId::new(*entity_id);
                let old_pos = self.entity_position(entity_id)?;
                let new_pos = old_pos
                    + EntityPositionDelta {
                        dx: *delta_x,
                        dy: *delta_y,
                        dz: *delta_z,
                    };
                self.register_entity_position(entity_id, new_pos);
                // Translate to absolute position update
                Some(Packet::TeleportEntity(TeleportEntity {
                    entity_id: entity_id.as_i32(),
                    x: new_pos.x,
                    y: new_pos.y,
                    z: new_pos.z,
                    yaw: new_pos.yaw,
                    pitch: new_pos.pitch,
                    on_ground: *on_ground,
                }))
            }
            Packet::RemoveEntities(packet) => {
                for &entity_id in &packet.entities {
                    self.unload_entity(EntityId::new(entity_id));
                }
                None
            }
            Packet::Respawn(_) => {
                self.clear_entities();
                None
            }
            _ => None,
        }
    }
}
