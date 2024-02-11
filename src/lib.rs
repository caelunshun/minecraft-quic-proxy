//! Tool to proxy a Minecraft connection over QUIC, with the aim
//! of reducing lag due to head-of-line blocking over unreliable connections.
//!
//! The proxied connection looks like this:
//! Modded Minecraft client => this Rust library (via JNI) translates to QUIC => gateway server translates back to TCP => destination server
//!
//! This library implements the middle two layers: the translation of a TCP
//! connection into QUIC.
//!
//! # Proxying process
//! A newly opened connection first contacts the gateway server over QUIC.
//! It opens a single bidirectional stream, called the _control stream_,
//! which is used to transmit metadata related to the proxying process (i.e. not Minecraft packets).
//! The client sends a message over the control stream indicating the destination server it wishes to connect to.
//! (An authentication payload is also transmitted, to prevent the gateway server from being used
//! by third parties as a DoS proxy.)
//!
//! The gateway then connects to the destination server over a standard TCP connection. From
//! here on, any newly opened QUIC streams between the client and the gateway will be interpreted
//! as Minecraft packet streams and will therefore be converted to the TCP representation on each end.
//!
//! Should the connection become encrypted, which happens whenever the server is online mode,
//! special support from the client is required to allow the gateway to read the encryption key.
//! In this case, it sends a message over the control stream indicating the encryption key.
//! Note that Minecraft encryption is only applied between the gateway and the destination. Over QUIC,
//! the much more secure TLS built into QUIC is used instead.

#![feature(error_generic_member_access)]
#![allow(dead_code)]

pub mod client;
mod control_stream;
mod entity_id;
pub mod gateway;
mod io_duplex;
mod packet_translation;
mod position;
mod protocol;
mod proxy;
mod sequence;
mod stream;
mod stream_allocation;
mod stream_priority;

pub use quinn;
use quinn::{IdleTimeout, TransportConfig, VarInt};
use std::time::Duration;

/// Gets the QUIC transport config for a proxied connection.
pub fn transport_config() -> TransportConfig {
    let mut config = TransportConfig::default();
    config
        .max_concurrent_uni_streams(VarInt::from_u32(16384))
        .max_idle_timeout(Some(
            IdleTimeout::try_from(Duration::from_secs(30)).unwrap(),
        ));
    config
}
