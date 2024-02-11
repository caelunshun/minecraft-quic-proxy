use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign};

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

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct EntityPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

/// Delta encoding for a new entity position.
/// Used in the UpdateEntityPosition packet family.
#[derive(Copy, Clone, Debug)]
pub struct EntityPositionDelta {
    pub dx: i16,
    pub dy: i16,
    pub dz: i16,
}

impl Add<EntityPositionDelta> for EntityPosition {
    type Output = EntityPosition;

    fn add(self, rhs: EntityPositionDelta) -> Self::Output {
        EntityPosition {
            x: self.x + convert_delta(rhs.dx),
            y: self.y + convert_delta(rhs.dy),
            z: self.z + convert_delta(rhs.dz),
            yaw: self.yaw,
            pitch: self.pitch,
        }
    }
}

impl AddAssign<EntityPositionDelta> for EntityPosition {
    fn add_assign(&mut self, rhs: EntityPositionDelta) {
        *self = *self + rhs;
    }
}

impl From<(f64, f64, f64, f32, f32)> for EntityPosition {
    fn from(value: (f64, f64, f64, f32, f32)) -> Self {
        let (x, y, z, pitch, yaw) = value;
        Self {
            x,
            y,
            z,
            pitch,
            yaw,
        }
    }
}

fn convert_delta(delta: i16) -> f64 {
    f64::from(delta) / 4096.0
}
