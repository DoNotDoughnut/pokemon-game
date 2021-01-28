use std::io::Cursor;

use parking_lot::Mutex;
use zip::ZipArchive as Archive;

pub mod map_serializable;

pub mod map_loader;
pub mod chunk_map_loader;
pub mod map_set_loader;

pub mod warp_loader;
pub mod npc_loader;
pub mod wild_entry_loader;

pub mod gba_map;
pub mod json_map {
    pub mod v1;
}

lazy_static::lazy_static! {
    pub static ref WORLD_ARCHIVE: Mutex<Archive<Cursor<&'static [u8; 543646]>>> = Mutex::new(Archive::new(Cursor::new(include_bytes!("../../../include/world.zip"))).expect("Could not read world archive in executable!")); 
}