use serde::{Deserialize, Serialize};

/// Wrapper for a Minecraft network entity ID.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(i32);

impl EntityId {
    pub fn new(id: i32) -> Self {
        Self(id)
    }

    pub fn as_i32(self) -> i32 {
        self.0
    }
}
