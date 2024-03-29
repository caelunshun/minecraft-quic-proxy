//! # QUIC stream allocation
//! The benefit of QUIC over TCP comes from the fact that data sent on separate streams
//! is not required to be received in order, thus reducing head-of-line blocking.
//! It would be valid for the proxy to transmit all packets over the same stream, but this
//! would accomplish nothing compared to TCP.
//!
//! When allocating packets to streams, care must be taken to ensure data that must
//! be received in order is guaranteed to be received in that order. Thus, sequentially related
//! packets must be transmitted on the same stream.
//!
//! The proxy will make the following choices regarding stream allocation. Note that
//! all opened streams are unidirectional.
//!
//! - During the Handshake, Status, Login, and Configuration stages of the connection, all
//!  packets are sent on the same stream.
//! - During the Play state:
//!   - All entity movement packets (including players) are sent as unreliable datagrams and tagged
//!      with an ordinal. Only a packet that has a greater ordinal than all previously received datagrams
//!      associated with that entity is used. Older datagrams are dropped.
//!   - Other packets sent for specific entities are sent on a stream belonging to that entity.
//!   - Packets updating blocks or chunks are sent on a stream belonging to that chunk.
//!   - Packets pertaining to chat use the chat stream.
//!   - The following packets use a new stream for each packet (i.e., reliable unordered):
//!       - Keepalives
//!       - Ping/pong
//!   - All other packets use the shared "miscellaneous" stream.

use crate::{
    entity_id::EntityId,
    position::ChunkPosition,
    protocol::{
        packet,
        packet::{
            client, server,
            server::play::EntityAnimation,
            side,
            side::{Client, Server},
            state,
        },
    },
    sequence::SequenceKey,
    stream::SendStreamHandle,
    stream_priority,
};
use mini_moka::sync::Cache;
use quinn::Connection;
use std::time::Duration;

/// Tells the proxy how to transmit a packet.
pub enum Allocation<Side: packet::Side> {
    /// The packet will be sent on the given stream
    /// (reliable, ordered only with respect to that stream)
    Stream(SendStreamHandle<Side, state::Play>),
    /// The packet should be sent as an unreliable datagram
    /// on the connection, with an ordinal allocated from
    /// the given sequence.
    /// (unreliable, unordered)
    UnreliableSequence(SequenceKey),
}

/// Stores all QUIC streams used for _transmitting_ packets on a connection.
///
/// Note that this is only used during the Play connection state. During the login/setup states,
/// all packets are simply sent on the same stream.
///
/// The `Side` generic parameter will be `Client` when used on the client,
/// and `Server` when used on the gateway.
///
/// # Implementation details
/// Some streams are keyed according to some sort of game identifier
/// (e.g., entity ID or chunk position). These streams can become stale
/// after their game entities are no longer alive / in sight.
///
/// To avoid a memory leak, streams can be automatically dropped
/// after being unused for `STREAM_IDLE_DURATION`. Technically,
/// this allows packets on the same logical stream to be received
/// out of order (if the stream corresponding to that entity was re-created
/// after the old one was dropped), but such situations are extremely
/// rare for sufficiently high idle duration.
pub struct StreamAllocator<Side: packet::Side> {
    connection: Connection,

    entity_streams: Cache<EntityId, SendStreamHandle<Side, state::Play>>,
    block_update_streams: Cache<ChunkPosition, SendStreamHandle<Side, state::Play>>,

    chunk_stream: SendStreamHandle<Side, state::Play>,
    chat_stream: SendStreamHandle<Side, state::Play>,
    misc_stream: SendStreamHandle<Side, state::Play>,
}

/// Minimum duration a stream must be kept with no activity.
pub const STREAM_IDLE_DURATION: Duration = Duration::from_secs(90);

impl<Side> StreamAllocator<Side>
where
    Side: packet::Side + Clone,
{
    pub async fn new(connection: &Connection) -> anyhow::Result<Self> {
        let chat_stream =
            SendStreamHandle::open(connection, "chat", stream_priority::CHAT_STREAM).await?;
        let misc_stream =
            SendStreamHandle::open(connection, "misc", stream_priority::MISC_STREAM).await?;
        let chunk_stream =
            SendStreamHandle::open(connection, "chunks", stream_priority::DEFAULT).await?;

        let entity_streams = Cache::builder().time_to_idle(STREAM_IDLE_DURATION).build();
        let block_update_streams = Cache::builder().time_to_idle(STREAM_IDLE_DURATION).build();
        Ok(Self {
            connection: connection.clone(),
            entity_streams,
            block_update_streams,
            chunk_stream,
            chat_stream,
            misc_stream,
        })
    }

    async fn block_update_stream(
        &self,
        chunk: ChunkPosition,
    ) -> anyhow::Result<SendStreamHandle<Side, state::Play>> {
        match self.block_update_streams.get(&chunk) {
            Some(stream) => Ok(stream.clone()),
            None => {
                let stream = SendStreamHandle::open(
                    &self.connection,
                    format!("{chunk:?}"),
                    stream_priority::GAME_UPDATES,
                )
                .await?;
                self.block_update_streams.insert(chunk, stream.clone());
                Ok(stream)
            }
        }
    }

    async fn entity_stream(
        &self,
        entity_id: EntityId,
    ) -> anyhow::Result<SendStreamHandle<Side, state::Play>> {
        match self.entity_streams.get(&entity_id) {
            Some(stream) => Ok(stream.clone()),
            None => {
                let stream = SendStreamHandle::open(
                    &self.connection,
                    "entity",
                    stream_priority::GAME_UPDATES,
                )
                .await?;
                self.entity_streams.insert(entity_id, stream.clone());
                Ok(stream)
            }
        }
    }
}

/// `StreamAllocator` implements this for both `Side = Client` and `Side = Server`
/// (the only two `Side` implementors).
pub trait AllocateStream<Side: packet::Side + 'static> {
    /// Allocates a stream for the given packet.
    async fn allocate_stream_for(
        &mut self,
        packet: &Side::SendPacket<state::Play>,
    ) -> anyhow::Result<Allocation<Side>>;
}

impl AllocateStream<side::Client> for StreamAllocator<side::Client> {
    async fn allocate_stream_for(
        &mut self,
        packet: &client::play::Packet,
    ) -> anyhow::Result<Allocation<Client>> {
        use client::play::Packet;

        let allocation = match packet {
            Packet::ChatCommand(_) | Packet::ChatMessage(_) | Packet::AcknowledgeMessage(_) => {
                Allocation::Stream(self.chat_stream.clone())
            }

            Packet::KeepAlive(_) | Packet::PingRequest(_) | Packet::Pong(_) => {
                let new_stream = SendStreamHandle::open(
                    &self.connection,
                    "keepalive",
                    stream_priority::KEEPALIVE,
                )
                .await?;
                Allocation::Stream(new_stream)
            }

            _ => Allocation::Stream(self.misc_stream.clone()),
        };
        Ok(allocation)
    }
}

impl AllocateStream<side::Server> for StreamAllocator<side::Server> {
    async fn allocate_stream_for(
        &mut self,
        packet: &server::play::Packet,
    ) -> anyhow::Result<Allocation<Server>> {
        use server::play::*;
        let allocation = match packet {
            // Chat stream
            Packet::ChatSuggestions(_)
            | Packet::DisguisedChatMessage(_)
            | Packet::PlayerChatMessage(_)
            | Packet::SystemChatMessage(_)
            | Packet::BossBar(_)
            | Packet::ClearTitles(_)
            | Packet::CommandSuggestions(_)
            | Packet::DeleteMessage(_)
            | Packet::SetActionBarText(_)
            | Packet::SetSubtitleText(_)
            | Packet::SetTitleText(_)
            | Packet::SetTitleAnimationTimes(_) => Allocation::Stream(self.chat_stream.clone()),

            // New stream (reliable unordered)
            Packet::Particle(_)
            | Packet::Explosion(_)
            | Packet::SoundEffect(_)
            | Packet::StopSound(_)
            | Packet::SetHealth(_)
            | Packet::KeepAlive(_)
            | Packet::Ping(_)
            | Packet::PingResponse(_) => {
                let new_stream = SendStreamHandle::open(
                    &self.connection,
                    "keepalive",
                    stream_priority::KEEPALIVE,
                )
                .await?;
                Allocation::Stream(new_stream)
            }

            // Chunk stream
            Packet::UnloadChunk(_)
            | Packet::ChunkAndLightData(_)
            | Packet::UpdateLight(_)
            | Packet::ChunkBatchFinished(_)
            | Packet::ChunkBatchStart(_)
            | Packet::ChunkBiomes(_) => Allocation::Stream(self.chunk_stream.clone()),

            // Block update streams (ordered on chunk)
            Packet::UpdateSectionBlocks(packet) => {
                Allocation::Stream(self.block_update_stream(packet.chunk_position()).await?)
            }
            Packet::BlockUpdate(packet) => {
                Allocation::Stream(self.block_update_stream(packet.position.chunk()).await?)
            }

            // Entity update streams (ordered on entity ID)
            Packet::EntityAnimation(EntityAnimation { entity_id, .. })
            | Packet::EntityEvent(EntityEvent { entity_id, .. })
            | Packet::HurtAnimation(HurtAnimation { entity_id, .. })
            | Packet::SetHeadRotation(SetHeadRotation { entity_id, .. })
            | Packet::EntityEffect(EntityEffect { entity_id, .. })
            | Packet::DamageEvent(DamageEvent { entity_id, .. }) => {
                Allocation::Stream(self.entity_stream(EntityId::new(*entity_id)).await?)
            }
            Packet::RemoveEntities(RemoveEntities { entities, .. }) if entities.len() == 1 => {
                // TODO: cover case where entities.len() > 1, likely by splitting the packet into multiple
                // RemoveEntities messages.
                Allocation::Stream(self.entity_stream(EntityId::new(entities[0])).await?)
            }

            // Unreliable entity datagrams
            Packet::UpdateEntityRotation(UpdateEntityRotation { entity_id, .. })
            | Packet::UpdateEntityPositionAndRotation(UpdateEntityPositionAndRotation {
                entity_id,
                ..
            })
            | Packet::UpdateEntityPosition(UpdateEntityPosition { entity_id, .. })
            | Packet::TeleportEntity(TeleportEntity { entity_id, .. }) => {
                Allocation::UnreliableSequence(SequenceKey::EntityPosition(EntityId::new(
                    *entity_id,
                )))
            }

            Packet::SetEntityVelocity(SetEntityVelocity { entity_id, .. }) => {
                Allocation::UnreliableSequence(SequenceKey::EntityVelocity(EntityId::new(
                    *entity_id,
                )))
            }

            // Default case - shared stream
            _ => Allocation::Stream(self.misc_stream.clone()),
        };
        Ok(allocation)
    }
}
