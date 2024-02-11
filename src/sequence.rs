use crate::{
    entity_id::EntityId,
    protocol::{packet, packet::state, Decode, Decoder, Encode, Encoder},
};
use anyhow::Context;
use bincode::Options;
use mini_moka::unsync::Cache;
use quinn::Connection;
use serde::{Deserialize, Serialize};
use std::{
    cell::{Cell, RefCell},
    marker::PhantomData,
    rc::Rc,
    thread,
    time::Duration,
};
use tokio::{sync::oneshot, task::LocalSet};

type SendPacket<Side> = (
    SequenceKey,
    <Side as packet::Side>::SendPacket<state::Play>,
    oneshot::Sender<anyhow::Result<()>>,
);

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
#[derive(Clone)]
pub struct SequencesHandle<Side: packet::Side> {
    sender: flume::Sender<SendPacket<Side>>,
    receiver: flume::Receiver<anyhow::Result<Side::RecvPacket<state::Play>>>,
}

/// Idle duration after which the state for a certain sequence
/// is dropped to conserve memory.
const SEQUENCE_IDLE_DURATION: Duration = Duration::from_secs(120);

impl<Side> SequencesHandle<Side>
where
    Side: packet::Side,
{
    pub fn new(connection: Connection) -> Self {
        let (packets_inbound_tx, packets_inbound_rx) = flume::bounded(16);
        let (packets_outbound_tx, packets_outbound_rx) = flume::bounded::<SendPacket<Side>>(16);

        let runtime = tokio::runtime::Handle::current();
        thread::spawn(move || {
            let local_set = LocalSet::new();
            let sequences = Rc::new(Sequences::<Side>::new(connection));

            local_set.spawn_local({
                let sequences = Rc::clone(&sequences);
                async move {
                    loop {
                        match sequences.recv_packet().await {
                            Ok(packet) => {
                                if packets_inbound_tx.send_async(Ok(packet)).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                packets_inbound_tx.send_async(Err(e)).await.ok();
                                break;
                            }
                        }
                    }
                }
            });
            local_set.spawn_local(async move {
                while let Ok((sequence_key, packet, completion)) =
                    packets_outbound_rx.recv_async().await
                {
                    let result = sequences.send_packet(sequence_key, packet).await;
                    let is_error = result.is_err();
                    completion.send(result).ok();
                    if is_error {
                        break;
                    }
                }
            });

            runtime.block_on(local_set);
        });

        Self {
            sender: packets_outbound_tx,
            receiver: packets_inbound_rx,
        }
    }

    pub async fn send_packet(
        &self,
        sequence_key: SequenceKey,
        packet: Side::SendPacket<state::Play>,
    ) -> anyhow::Result<()> {
        let (completion_tx, completion_rx) = oneshot::channel();
        self.sender
            .send_async((sequence_key, packet, completion_tx))
            .await
            .ok()
            .context("disconnected")?;
        completion_rx.await.context("disconnected")??;
        Ok(())
    }

    pub async fn recv_packet(&self) -> anyhow::Result<Side::RecvPacket<state::Play>> {
        self.receiver.recv_async().await.context("disconnected")?
    }
}

struct Sequences<Side> {
    connection: Connection,
    sequences: RefCell<Cache<SequenceKey, Rc<Sequence>>>,
    _marker: PhantomData<Side>,
}

impl<Side> Sequences<Side>
where
    Side: packet::Side,
{
    pub fn new(connection: Connection) -> Self {
        Self {
            connection,
            sequences: RefCell::new(
                Cache::builder()
                    .time_to_idle(SEQUENCE_IDLE_DURATION)
                    .build(),
            ),
            _marker: PhantomData,
        }
    }

    /// Sends a packet on the given sequence.
    pub async fn send_packet(
        &self,
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
    pub async fn recv_packet(&self) -> anyhow::Result<Side::RecvPacket<state::Play>> {
        loop {
            let datagram = self.connection.read_datagram().await?;
            let (header, packet) = self.decode_packet(&datagram)?;
            let sequence = self.get_sequence(header.key);
            if sequence.receive_packet(header.ordinal) {
                return Ok(packet);
            }
        }
    }

    fn get_sequence(&self, key: SequenceKey) -> Rc<Sequence> {
        let mut sequences = self.sequences.borrow_mut();
        if let Some(sequence) = sequences.get(&key) {
            return Rc::clone(sequence);
        }

        sequences.insert(key, Rc::new(Sequence::new()));
        Rc::clone(sequences.get(&key).unwrap())
    }

    /// Encodes a packet to its datagram representation,
    /// using the given ordinal and sequence key.
    fn encode_packet(
        &self,
        packet: &impl Encode,
        header: DatagramHeader,
    ) -> anyhow::Result<Vec<u8>> {
        let mut buf = bincode::options()
            .allow_trailing_bytes()
            .serialize(&header)?;
        packet.encode(&mut Encoder::new(&mut buf));
        Ok(buf)
    }

    /// Decodes a packet from its datagram representation, using the given
    /// ordinal and sequence key.
    fn decode_packet<P: Decode>(&self, mut bytes: &[u8]) -> anyhow::Result<(DatagramHeader, P)> {
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
    EntityPosition(EntityId),
    EntityVelocity(EntityId),

    /// The player entity - used for serverbound position updates.
    ThePlayerPosition,
}
