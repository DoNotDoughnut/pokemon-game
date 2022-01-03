use serde::{Deserialize, Serialize};

use hashbrown::HashMap;

use crate::{
    character::{
        npc::{group::NpcGroupId, Npc, NpcId},
        Movement,
    },
    map::{manager::WorldMapData, PaletteId, TileId, object::ObjectId},
    // positions::Location,
};

// pub type MapGuiLocs = HashMap<crate::map::MapIcon, (String, Location)>;

type Texture = Vec<u8>;

#[derive(Deserialize, Serialize)]
pub struct SerializedWorld {
    pub data: WorldMapData,
    // pub map_gui_locs: MapGuiLocs,
    pub textures: SerializedTextures,
}

#[derive(Deserialize, Serialize)]
pub struct SerializedNpc {
    pub id: NpcId,
    pub npc: Npc,
}

pub type SerializedPaletteMap = HashMap<PaletteId, SerializedPalette>;
pub type SerializedPlayerTexture = HashMap<Movement, Texture>;
pub type SerializedNpcGroupTextures = HashMap<NpcGroupId, Texture>;
pub type SerializedObjectTextures = HashMap<ObjectId, Texture>;

#[derive(Deserialize, Serialize)]
pub struct SerializedTextures {
    pub palettes: SerializedPaletteMap,
    pub npcs: SerializedNpcGroupTextures,
    pub objects: SerializedObjectTextures,
    pub player: SerializedPlayerTexture,
}

pub type SerializedAnimatedTexture = HashMap<TileId, Texture>;
pub type SerializedDoors = HashMap<TileId, Texture>;

#[derive(Deserialize, Serialize)]
pub struct SerializedPalette {
    pub texture: Texture,
    pub animated: SerializedAnimatedTexture,
    pub doors: SerializedDoors,
}
