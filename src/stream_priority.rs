//! Constants used for different stream priorities.

pub const DEFAULT: i32 = 0;

pub const MISC_STREAM: i32 = 5;

pub const CHAT_STREAM: i32 = 6;
pub const GAME_UPDATES: i32 = 7;

/// Keepalives keep the connection alive, prioritize them
pub const KEEPALIVE: i32 = 10;
