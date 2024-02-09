use crate::{
    entity_id::EntityId,
    position::ChunkPosition,
    protocol::{packet, packet::state, Decode, Decoder, Encode, Encoder},
};
use bincode::Options;
use mini_moka::unsync::Cache;
use quinn::Connection;
use serde::{Deserialize, Serialize};
use std::{cell::Cell, marker::PhantomData, rc::Rc, time::Duration};

/// Manages sending and receiving sequenced datagrams.
/// Sequenced datagrams are associated with a particular
/// sequence, mapped by a `SequenceKey`.
///
/// Sequenced packets are sent unreliably and unordered,
/// and without any compression.
/// However, the sequence logic adds one detail on top:
/// each packet is assigned an index / ordinal in its sequence.
/// When a packet is received, if its ordinal number is less
/// than the previously received packet, it is ignored.
///
/// This allows only the newest received packet to be considered.
pub struct Sequences<Side> {
    connection: Connection,
    sequences: Cache<SequenceKey, Rc<Sequence>>,
    _marker: PhantomData<Side>,
}

/// Idle duration after which the state for a certain sequence
/// is dropped to conserve memory.
const SEQUENCE_IDLE_DURATION: Duration = Duration::from_secs(120);

impl<Side> Sequences<Side>
where
    Side: packet::Side,
{
    pub fn new(connection: Connection) -> Self {
        Self {
            connection,
            sequences: Cache::builder()
                .time_to_idle(SEQUENCE_IDLE_DURATION)
                .build(),
            _marker: PhantomData,
        }
    }

    /// Sends a packet on the given sequence.
    pub async fn send_packet(
        &mut self,
        sequence_key: SequenceKey,
        packet: Side::SendPacket<state::Play>,
    ) -> anyhow::Result<()> {
        let sequence = self.get_sequence(sequence_key);
        let ordinal = sequence.next_send_ordinal();
        let bytes = self.encode_packet(
            &packet,
            DatagramHeader {
                ordinal,
                key: sequence_key,
            },
        )?;
        self.connection.send_datagram(bytes.into())?;
        Ok(())
    }

    /// Waits for the next datagram.
    /// Ignores any out-of-date packets, as per the sequence logic.
    pub async fn recv_packet(&mut self) -> anyhow::Result<Side::RecvPacket<state::Play>> {
        loop {
            let datagram = self.connection.read_datagram().await?;
            let (header, packet) = self.decode_packet(&datagram)?;
            let sequence = self.get_sequence(header.key);
            if sequence.receive_packet(header.ordinal) {
                return Ok(packet);
            }
        }
    }

    fn get_sequence(&mut self, key: SequenceKey) -> Rc<Sequence> {
        if let Some(sequence) = self.sequences.get(&key) {
            return Rc::clone(sequence);
        }

        self.sequences.insert(key, Rc::new(Sequence::new()));
        Rc::clone(self.sequences.get(&key).unwrap())
    }

    /// Encodes a packet to its datagram representation,
    /// using the given ordinal and sequence key.
    fn encode_packet(
        &mut self,
        packet: &impl Encode,
        header: DatagramHeader,
    ) -> anyhow::Result<Vec<u8>> {
        let mut buf = bincode::options().serialize(&header)?;
        packet.encode(&mut Encoder::new(&mut buf));
        Ok(buf)
    }

    /// Decodes a packet from its datagram representation, using the given
    /// ordinal and sequence key.
    fn decode_packet<P: Decode>(
        &mut self,
        mut bytes: &[u8],
    ) -> anyhow::Result<(DatagramHeader, P)> {
        // Note: passing `&mut bytes` as the reader here
        // advances the `bytes` slice past the end of the header,
        // allowing us to decode the packet contents afterward.
        let header: DatagramHeader = bincode::options()
            .allow_trailing_bytes()
            .deserialize_from(&mut bytes)?;

        let packet = P::decode(&mut Decoder::new(bytes))?;
        Ok((header, packet))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DatagramHeader {
    key: SequenceKey,
    ordinal: u64,
}

struct Sequence {
    send_counter: Cell<u64>,
    newest_received: Cell<u64>,
}

impl Sequence {
    pub fn new() -> Self {
        Self {
            send_counter: Cell::new(0),
            newest_received: Cell::new(0),
        }
    }

    pub fn next_send_ordinal(&self) -> u64 {
        let ordinal = self.send_counter.get();
        self.send_counter.set(ordinal.wrapping_add(1));
        ordinal
    }

    /// Called when a datagram is received.
    /// Returns whether the packet should be kept (`true`) or dropped (`false`).
    pub fn receive_packet(&self, packet_ordinal: u64) -> bool {
        // use `>=` to handle the initial case where ordinal == 0
        if packet_ordinal >= self.newest_received.get() {
            self.newest_received.set(packet_ordinal);
            true
        } else {
            false
        }
    }
}

/// Key value used for sequenced, unreliable-unordered datagrams.
/// Packets with the same key are sent on the same sequence.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SequenceKey {
    Entity(EntityId),
    Chunk(ChunkPosition),
}
