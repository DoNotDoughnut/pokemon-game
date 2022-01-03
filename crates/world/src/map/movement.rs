use crate::positions::Direction;

use super::chunk::{ChunkOffset, Connection};

pub type MovementId = u8;
pub type Elevation = Option<u8>;

pub enum MapMovementResult<'a> {
    Option(Option<MovementId>),
    /// Second argument is for offset
    Chunk(Direction, ChunkOffset, &'a [Connection]),
}

impl MapMovementResult<'_> {
    pub const NONE: Self = Self::Option(None);
}

impl<'a> From<Option<MovementId>> for MapMovementResult<'a> {
    fn from(id: Option<MovementId>) -> Self {
        Self::Option(id)
    }
}

impl<'a> From<Option<(&'a Direction, i32, &'a [Connection])>> for MapMovementResult<'a> {
    fn from(connection: Option<(&'a Direction, i32, &'a [Connection])>) -> Self {
        match connection {
            Some((direction, offset, connection)) => {
                Self::Chunk(*direction, offset as _, connection)
            }
            None => Self::Option(None),
        }
    }
}

impl super::WorldMap {
    pub const CROSSING: MovementId = 0x0;
    pub const OBSTACLE: MovementId = 0x1;
    /// Height level 0 is used for water.
    pub const WATER: MovementId = 0x4;
    /// Height level 0 obstacle
    pub const WATER_OBSTACLE: MovementId = 0x5;
    pub const HL1: MovementId = 0x8;
    pub const HL1_OBSTACLE: MovementId = 0x9;
    pub const HL2: MovementId = 0xC;
    pub const HL2_OBSTACLE: MovementId = 0xD;
    pub const HL3: MovementId = 0x10;
    pub const HL3_OBSTACLE: MovementId = 0x11;
    pub const HL4: MovementId = 0x14;
    pub const HL4_OBSTACLE: MovementId = 0x15;

    pub const fn can_move(elevation: Elevation, code: MovementId) -> bool {
        match elevation {
            None => code % 2 == 0,
            Some(elevation) => {
                if code == 0 {
                    return true;
                }
                match elevation {
                    0 | 2 => matches!(code, Self::WATER | Self::HL2),
                    1 => code == Self::HL1,
                    3 => code == Self::HL3,
                    4 => code == Self::HL4,
                    _ => false,
                }
            }
        }
    }

    pub fn change_elevation(elevation: &mut Elevation, code: MovementId) {
        *elevation = match code {
            Self::CROSSING => None,
            Self::WATER => Some(0),
            Self::HL1 => Some(1),
            Self::HL2 => Some(2),
            Self::HL3 => Some(3),
            Self::HL4 => Some(4),
            _ => return,
        }
    }
}
