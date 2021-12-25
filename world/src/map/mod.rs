use serde::{Deserialize, Serialize};

use crate::{
    character::npc::Npcs,
    positions::{Coordinate, CoordinateInt, Direction, Location},
};
use warp::{WarpDestination, Warps};
use wild::WildEntries;

use self::{chunk::WorldChunk, movement::MapMovementResult};

pub mod chunk;
pub mod movement;

pub mod manager;
pub mod warp;
pub mod wild;

pub mod battle;

pub type TileId = u16;
pub type MapSize = usize;
pub type PaletteId = u8;
pub type Palettes = [PaletteId; 2];
pub type MovementId = movement::MovementId;
pub type MusicId = tinystr::TinyStr16;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMap {
    pub id: Location,

    pub name: String,
    pub music: MusicId,

    pub width: CoordinateInt,
    pub height: CoordinateInt,

    pub palettes: Palettes,

    pub tiles: Vec<TileId>,
    pub movements: Vec<MovementId>,

    pub border: [TileId; 4], //Border, // border blocks

    pub chunk: Option<WorldChunk>,

    // Map objects
    pub warps: Warps,

    pub wild: Option<WildEntries>,

    pub npcs: Npcs,

    // pub objects: HashMap<u8, MapObject>,
    // pub scripts: Vec<WorldScript>,

    #[serde(default)]
    pub settings: WorldMapSettings,
    // pub mart: Option<mart::Pokemart>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorldMapSettings {
    #[serde(default)]
    /// To - do: rename to "fly"
    pub fly_position: Option<Coordinate>,
    #[serde(default)]
    pub brightness: Brightness,
}

impl WorldMap {
    pub fn in_bounds(&self, coords: Coordinate) -> bool {
        !(coords.x.is_negative()
            || coords.x >= self.width
            || coords.y.is_negative()
            || coords.y >= self.height)
    }

    pub fn tile(&self, coords: Coordinate) -> Option<TileId> {
        self.in_bounds(coords)
            .then(|| self.tiles[coords.x as usize + coords.y as usize * self.width as usize])
    }

    pub fn local_movement(&self, coords: Coordinate) -> Option<MovementId> {
        self.in_bounds(coords)
            .then(|| self.unbounded_movement(coords))
            .flatten()
    }

    pub fn unbounded_movement(&self, coords: Coordinate) -> Option<MovementId> {
        self.movements
            .get(coords.x as usize + coords.y as usize * self.width as usize)
            .map(|code| {
                match self
                    .npcs
                    .values()
                    .any(|npc| npc.character.position.coords == coords)
                {
                    true => 1,
                    false => *code,
                }
            })
    }

    pub fn chunk_movement(&self, coords: Coordinate) -> MapMovementResult {
        if let Some(chunk) = self.chunk.as_ref() {
            if coords.x.is_negative() {
                return chunk
                    .connections
                    .get_key_value(&Direction::Left)
                    .map(|(d, c)| (d, coords.y, c.as_ref()))
                    .into();
            }

            if coords.x >= self.width {
                return chunk
                    .connections
                    .get_key_value(&Direction::Right)
                    .map(|(d, c)| (d, coords.y, c.as_ref()))
                    .into();
            }

            if coords.y.is_negative() {
                return chunk
                    .connections
                    .get_key_value(&Direction::Up)
                    .map(|(d, c)| (d, coords.x, c.as_ref()))
                    .into();
            }

            if coords.y >= self.height {
                return chunk
                    .connections
                    .get_key_value(&Direction::Down)
                    .map(|(d, c)| (d, coords.x, c.as_ref()))
                    .into();
            }
        } else if !self.in_bounds(coords) {
            return MapMovementResult::NONE;
        }
        self.unbounded_movement(coords).into()
    }

    pub fn warp_at(&self, coords: Coordinate) -> Option<&WarpDestination> {
        self.warps
            .values()
            .find(|warp| warp.location.in_bounds(&coords))
            .map(|entry| &entry.destination)
    }

    pub fn contains(&self, location: &Location) -> bool {
        &self.id == location
            || self
                .chunk
                .as_ref()
                .map(|chunk| {
                    chunk
                        .connections
                        .values()
                        .flatten()
                        .any(|connection| &connection.0 == location)
                })
                .unwrap_or_default()
    }
}

/// To - do: move
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum Brightness {
    Day,
    Night,
    // FlashNeeded,
}

// #[deprecated]
// #[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
// pub enum MapIcon {
//     City(u8, u8),
//     Route(u8, u8, u8, u8),
// }

impl Default for Brightness {
    fn default() -> Self {
        Self::Day
    }
}
