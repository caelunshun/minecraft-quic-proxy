//! Constants used for different stream priorities.

pub const DEFAULT: i32 = 0;

/// Misc stream takes precedence over others (e.g. chunk stream)
pub const MISC_STREAM: i32 = 5;

pub const CHAT_STREAM: i32 = 6;

/// Keepalives keep the connection alive, prioritize them
pub const KEEPALIVE: i32 = 10;
