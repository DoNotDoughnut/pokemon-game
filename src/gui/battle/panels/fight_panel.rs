use crate::battle::battle_pokemon::BattlePokemon;
use crate::util::Entity;
use crate::gui::Focus;
use crate::io::input::Control;
use crate::util::graphics::Texture;

use crate::gui::GuiComponent;
use crate::io::input;

use crate::util::graphics::draw;
use crate::util::graphics::texture::byte_texture;

use super::move_panel::MovePanel;
pub struct FightPanel {

    active: bool,
    focus: bool,

    x: f32,
    y: f32,
    panel_x: f32,
    panel_y: f32,

    background: Texture,
    move_panel: MovePanel,

    pub cursor_position: u8,
    pub next: u8,

    move_names: Vec<String>,

}

impl FightPanel {

    pub fn new(panel_x: f32, panel_y: f32) -> FightPanel {

        let x = 0.0;
        let y = 0.0;

        FightPanel {

            active: false,
            focus: false,

            x: x,
            y: y,
            panel_x: panel_x,
            panel_y: panel_y,

            background: byte_texture(include_bytes!("../../../../build/assets/gui/battle/move_panel.png")),
            move_panel: MovePanel::new(x + panel_x, y + panel_y),

            cursor_position: 0,
            next: 0,

            move_names: Vec::new(),

        }

    }

    pub fn update_names(&mut self, instance: &BattlePokemon) {
        self.move_names = Vec::new();
        for move_instance in &instance.moves {
            self.move_names.push(move_instance.pokemon_move.name.clone());            
        }
    }

    fn reset_vars(&mut self) {
        self.next = 0;
        self.cursor_position = 0;
    }

}

impl GuiComponent for FightPanel {

    fn update_position(&mut self, panel_x: f32, panel_y: f32) {
        self.panel_x = panel_x;
        self.panel_y = panel_y;
    }

    fn render(&self) {
        if self.is_alive() {

            draw(self.background, self.x + self.panel_x, self.y + self.panel_y);
            self.move_panel.render();
        
            for string_id in 0..self.move_names.len() {
                let mut x_offset = 0.0;
                let mut y_offset = 0.0;
                if string_id % 2 == 1 {
                    x_offset = 72.0;
                }
                if string_id / 2 == 1 {
                    y_offset = 17.0;
                }
                crate::util::graphics::draw_text_left_color(0, self.move_names[string_id].to_ascii_uppercase().as_str(), crate::io::data::text::color::TextColor::Black, self.panel_x + self.x + 16.0 + x_offset, self.panel_y + self.y + 8.0 + y_offset);
                if string_id == self.cursor_position as usize {
                    crate::util::graphics::draw_cursor(self.panel_x + self.x + 10.0 + x_offset, self.panel_y + self.y + 10.0 + y_offset);
                }
            }
        }        
    }

}

impl crate::util::Input for FightPanel {

    fn input(&mut self, _delta: f32) {
        if input::pressed(Control::B) {
            self.next = 1;
        }
        if input::pressed(Control::Up) {
            if self.cursor_position >= 2 {
                self.cursor_position -= 2;
            }            
        } else if input::pressed(Control::Down) {
            if self.cursor_position < 2 {
                self.cursor_position += 2;
            } 
        } else if input::pressed(Control::Left) {
            if self.cursor_position > 0 {
                self.cursor_position -= 1;
            }
        } else if input::pressed(Control::Right) {
            if self.cursor_position < 3 {
                self.cursor_position += 1;
            }
        }
        if self.cursor_position >= self.move_names.len() as u8 {
            if self.cursor_position == 0 {
                self.cursor_position = 0;
            } else {
                self.cursor_position = self.move_names.len() as u8 - 1;
            }
            
        }
    }

}

impl Entity for FightPanel {

    fn spawn(&mut self) {
        self.active = true;
        self.move_panel.spawn();
        self.reset_vars();
    }

    fn despawn(&mut self) {
        self.active = false;
        self.move_panel.despawn();
        self.unfocus();
    }

    fn is_alive(& self) -> bool {
        self.active
    }

}

impl Focus for FightPanel {

    fn focus(&mut self) {
        self.focus = true;
    }

    fn unfocus(&mut self) {
        self.focus = false;
    }

    fn in_focus(&mut self) -> bool {
        self.focus
    }

}