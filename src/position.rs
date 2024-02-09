use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkPosition {
    pub x: i32,
    pub z: i32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPosition {
    pub fn chunk(self) -> ChunkPosition {
        ChunkPosition {
            x: self.x.rem_euclid(16),
            z: self.z.rem_euclid(16),
        }
    }
}
