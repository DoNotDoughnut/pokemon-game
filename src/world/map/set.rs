use firecore_world::map::set::WorldMapSet;
use firecore_world::map::set::manager::WorldMapSetManager;
use macroquad::prelude::warn;

use crate::battle::data::BattleData;
use crate::world::NPCTypes;
use crate::world::{GameWorld, TileTextures, NpcTextures, GuiTextures, RenderCoords};
use crate::world::gui::text_window::TextWindow;
use firecore_world::character::player::PlayerCharacter;

impl GameWorld for WorldMapSet {

    fn on_start(&self, music: bool) {
        self.map().on_start(music);
    }

    fn on_tile(&mut self, battle_data: &mut Option<BattleData>, player: &mut PlayerCharacter) {
        self.maps[self.current_map].on_tile(battle_data, player)
    }

    fn update(&mut self, delta: f32, player: &mut PlayerCharacter, battle_data: &mut Option<BattleData>, text_window: &mut TextWindow, npc_types: &NPCTypes) {
        self.maps[self.current_map].update(delta, player, battle_data, text_window, npc_types);
    }

    fn render(&self, tile_textures: &TileTextures, npc_textures: &NpcTextures, npc_types: &NPCTypes, gui_textures: &GuiTextures, screen: RenderCoords, border: bool) {
        self.maps[self.current_map].render(tile_textures, npc_textures, npc_types, gui_textures, screen, border)
    }

    fn input(&mut self, delta: f32, player: &mut PlayerCharacter) {
        self.maps[self.current_map].input(delta, player)
    }

}

impl GameWorld for WorldMapSetManager {

    fn on_start(&self, music: bool) {
        match self.map_sets.get(&self.current_map_set) {
            Some(map_set) => map_set.on_start(music),
            None => {
                warn!("Could not get current map set {}!", self.current_map_set);
            },
        }
    }

    fn on_tile(&mut self, battle_data: &mut Option<BattleData>, player: &mut PlayerCharacter) {
        self.map_set_mut().on_tile(battle_data, player)
    }

    fn update(&mut self, delta: f32, player: &mut PlayerCharacter, battle_data: &mut Option<BattleData>, text_window: &mut TextWindow, npc_types: &NPCTypes) {
        self.map_set_mut().update(delta, player, battle_data, text_window, npc_types);
    }

    fn render(&self, tile_textures: &TileTextures, npc_textures: &NpcTextures, npc_types: &NPCTypes, gui_textures: &GuiTextures, screen: RenderCoords, border: bool) {
        match self.map_sets.get(&self.current_map_set) {
            Some(map_set) => map_set.render(tile_textures, npc_textures, npc_types, gui_textures, screen, border),
            None => {
                warn!("Could not get current map set {}!", self.current_map_set);
            }
        }
    }

    fn input(&mut self, delta: f32, player: &mut PlayerCharacter) {
        self.map_set_mut().input(delta, player)
    }

}