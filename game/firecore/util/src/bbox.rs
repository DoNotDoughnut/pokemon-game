use serde::{Deserialize, Serialize};

use crate::Coordinate;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct BoundingBox {
    pub min: Coordinate,
    pub max: Coordinate,
}

impl BoundingBox {

    pub const fn in_bounds(&self, coordinate: &Coordinate) -> bool{
        if coordinate.x >= self.min.x && coordinate.x <= self.max.x {
            return coordinate.y >= self.min.y && coordinate.y <= self.max.y;
        } else {
            return false;
        }
    }

}