use std::{
    iter::Rev,
    ops::{Range, RangeInclusive},
};

use crate::engine::{
    math::{IVec2, Vec2},
    utils::{HEIGHT, WIDTH},
};

use crate::worldlib::{character::Character, positions::CoordinateInt, TILE_SIZE};

#[derive(Debug, Clone)]
pub struct RenderCoords {
    pub x: RangeInclusive<CoordinateInt>,
    pub y: Rev<Range<CoordinateInt>>,

    pub focus: Vec2,

    pub offset: IVec2,
}

const HALF_WIDTH: i32 = (WIDTH as i32 + TILE_SIZE as i32) >> 1;
const HALF_HEIGHT: i32 = (HEIGHT as i32 + TILE_SIZE as i32) >> 1;

const HALF_WIDTH_TILE: i32 = HALF_WIDTH >> 4;
const HALF_HEIGHT_TILE: i32 = (HALF_HEIGHT >> 4) + 2;

impl RenderCoords {
    pub fn new(character: &Character) -> Self {
        let coords = character.position.coords;

        Self {
            x: coords.x - HALF_WIDTH_TILE..=coords.x + HALF_WIDTH_TILE,
            y: (coords.y - HALF_HEIGHT_TILE..coords.y + HALF_HEIGHT_TILE).rev(),

            #[deprecated(
                note = "rounding may fix problem of black spaces between tiles while moving"
            )]
            focus: Vec2::new(
                ((coords.x + 1) << 4) as f32 + character.offset.x - HALF_WIDTH as f32,
                ((coords.y + 1) << 4) as f32 + character.offset.y - HALF_HEIGHT as f32,
            )
            .round(),

            offset: Default::default(),
        }
    }

    pub fn offset(&self, offset: IVec2) -> RenderCoords {
        // return offset x & y
        RenderCoords {
            offset,
            ..self.clone()
        }
    }
}
