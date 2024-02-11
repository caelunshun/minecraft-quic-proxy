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
    protocol::{
        packet,
        packet::{
            client, server,
            server::play::{
                SetEntityVelocity, TeleportEntity, UpdateEntityPosition,
                UpdateEntityPositionAndRotation, UpdateEntityRotation,
            },
            side,
            side::{Client, Server},
            state,
        },
    },
    sequence::SequenceKey,
    stream::SendStreamHandle,
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
        let chunk_stream = SendStreamHandle::open(connection, "chunks").await?;
        let chat_stream = SendStreamHandle::open(connection, "chat").await?;
        let misc_stream = SendStreamHandle::open(connection, "misc").await?;

        let entity_streams = Cache::builder().time_to_idle(STREAM_IDLE_DURATION).build();
        Ok(Self {
            connection: connection.clone(),
            entity_streams,
            chunk_stream,
            chat_stream,
            misc_stream,
        })
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
                let new_stream = SendStreamHandle::open(&self.connection, "keepalive").await?;
                Allocation::Stream(new_stream)
            }

            Packet::SetPlayerOnGround(_)
            | Packet::SetPlayerPosition(_)
            | Packet::SetPlayerRotation(_)
            | Packet::SetPlayerPositionAndRotation(_) => {
                Allocation::UnreliableSequence(SequenceKey::ThePlayer)
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
        use server::play::Packet;
        let allocation = match packet {
            // Chat stream
            Packet::ChatSuggestions(_)
            | Packet::DisguisedChatMessage(_)
            | Packet::PlayerChatMessage(_)
            | Packet::SystemChatMessage(_) => Allocation::Stream(self.chat_stream.clone()),

            // New stream (reliable unordered)
            Packet::Particle(_)
            | Packet::KeepAlive(_)
            | Packet::Ping(_)
            | Packet::PingResponse(_) => {
                let new_stream = SendStreamHandle::open(&self.connection, "keepalive").await?;
                Allocation::Stream(new_stream)
            }

            // Unreliable entity datagrams
            Packet::UpdateEntityRotation(UpdateEntityRotation { entity_id, .. })
            | Packet::UpdateEntityPositionAndRotation(UpdateEntityPositionAndRotation {
                entity_id,
                ..
            })
            | Packet::UpdateEntityPosition(UpdateEntityPosition { entity_id, .. })
            | Packet::TeleportEntity(TeleportEntity { entity_id, .. })
            | Packet::SetEntityVelocity(SetEntityVelocity { entity_id, .. }) => {
                Allocation::UnreliableSequence(SequenceKey::Entity(EntityId::new(*entity_id)))
            }

            // Default case - shared stream
            _ => Allocation::Stream(self.misc_stream.clone()),
        };
        Ok(allocation)
    }
}
