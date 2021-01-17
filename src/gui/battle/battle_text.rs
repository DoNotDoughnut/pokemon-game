use crate::util::context::GameContext;
use crate::util::text_renderer::TextRenderer;
use opengl_graphics::GlGraphics;
use piston_window::Context;

//use crate::battle::battle_manager::BattleManager;

use crate::battle::battle::Battle;
use crate::entity::Entity;
use crate::gui::gui::{GuiComponent, GuiText};
use crate::util::timer::Timer;

use super::battle_gui::BattleGui;
use super::pokemon_gui::PokemonGui;

pub fn pmove(battle: &mut Battle, battle_gui: &mut BattleGui) {
    if battle.pmove_queued {
        if battle_gui.battle_text.is_active() {
            if battle_gui.battle_text.can_continue {
                if !battle_gui.opponent_pokemon_gui.health_bar.is_moving()
                    && battle_gui.battle_text.timer.is_finished()
                {
                    battle.pmove_queued = false;
                    battle_gui.battle_text.disable();

                    if battle.opponent().current_hp == 0 {
                        battle.faint_queued = true;
                        battle.omove_queued = false;
                    }
                } else if !battle_gui.battle_text.timer.is_alive() {
                    battle_gui.battle_text.timer.spawn();
                    battle_gui
                        .opponent_pokemon_gui
                        .health_bar
                        .update_bar(battle.opponent().current_hp, battle.opponent().base.hp);
                }
                battle_gui.battle_text.timer.update();
            }
        } else {
            battle.player_move();
            battle_gui.battle_text.enable();
            battle_gui
                .battle_text
                .update_text(&battle.player().pokemon.data.name, &battle.player_move.name);
        }
    } else if battle.faint_queued {
        faint_queued(battle, battle_gui);
    } else if battle.omove_queued {
        omove(battle, battle_gui);
    }
}

pub fn omove(battle: &mut Battle, battle_gui: &mut BattleGui) {
    if battle.omove_queued {
        if battle_gui.battle_text.is_active() {
            if battle_gui.battle_text.can_continue {
                if !battle_gui.player_pokemon_gui.health_bar.is_moving()
                    && battle_gui.battle_text.timer.is_finished()
                {
                    battle.omove_queued = false;
                    battle_gui.battle_text.disable();

                    if battle.player().current_hp == 0 {
                        battle.faint_queued = true;
                        battle.pmove_queued = false;
                    }
                } else if !battle_gui.battle_text.timer.is_alive() {
                    battle_gui.battle_text.timer.spawn();
                    battle_gui
                        .player_pokemon_gui
                        .update_hp(battle.player().current_hp, battle.player().base.hp);
                }
                battle_gui.battle_text.timer.update();
            }
        } else {
            battle.opponent_move();
            battle_gui.battle_text.enable();
            battle_gui
                .battle_text
                .update_text(&battle.opponent().pokemon.data.name, &battle.opponent_move.name);
        }
    } else if battle.faint_queued {
        faint_queued(battle, battle_gui);
    } else if battle.pmove_queued {
        pmove(battle, battle_gui);
    }
}

fn faint_queued(battle: &mut Battle, battle_gui: &mut BattleGui) {
    if battle.player().current_hp == 0 {
        if battle_gui.battle_text.is_active() {
            if battle_gui.battle_text.can_continue {
                if battle_gui.battle_text.timer.is_finished() {
                    battle_gui.battle_text.disable();
                    battle.faint_queued = false;
                    battle.faint = true;
                } else if !battle_gui.battle_text.timer.is_alive() {
                    battle_gui.battle_text.timer.spawn();
                }
                battle_gui.battle_text.timer.update();
            }
        } else {
            battle_gui.battle_text.enable();
            battle_gui
                .battle_text
                .update_faint(&battle.player().pokemon.data.name);
        }
    } else {
        if battle_gui.battle_text.is_active() {
            if battle_gui.battle_text.can_continue {
                if battle_gui.battle_text.timer.is_finished() {
                    battle.faint_queued = false;
                    battle_gui.battle_text.disable();
                    battle.faint = true;
                } else if !battle_gui.battle_text.timer.is_alive() {
                    battle_gui.battle_text.timer.spawn();
                }
                battle_gui.battle_text.timer.update();
            }
        } else {
            battle_gui.battle_text.enable();
            battle_gui
                .battle_text
                .update_faint(&battle.opponent().pokemon.data.name);
        }
    }
}

// fn move_with_text(battle: &mut Battle, battle_text: &mut BattleText, user: &mut PokemonInstance, user_move_bool: &mut bool, user_health_bar: &mut HealthBar, user_health_text: Option<&mut BasicText>) {
// 	if battle_text.can_continue {
// 		if !user_health_bar.is_moving() && battle_text.timer.is_finished() {

// 			//user_move_bool = &mut false;
// 			battle_text.disable();

// 			if user.current_hp == 0 {
// 				battle.queue_faint();
// 			}

// 		} else if !battle_text.timer.is_alive() {
// 			battle_text.timer.spawn();
// 			user_health_bar.update_bar(user.current_hp, user.base.hp);
// 			if let Some(text) = user_health_text {
// 				let mut ch = user.current_hp.to_string();
// 				ch.push('/');
// 				ch.push_str(user.base.hp.to_string().as_str());
// 				text.text = ch;
// 			}
// 		}
// 		battle_text.update();
// 	}
// }

pub struct BattleText {
    alive: bool,

    x: isize,
    y: isize,
    panel_x: isize,
    panel_y: isize,

    pub text: Vec<String>,
    current_line: usize,
    pub font_id: usize,

    counter: u16,

    pub can_continue: bool,

    pub timer: Timer,
}

impl BattleText {
    pub fn new(_panel_x: isize, _panel_y: isize) -> BattleText {
        BattleText {
            alive: false,

            x: 11,
            y: 11,
            panel_x: _panel_x,
            panel_y: _panel_y,

            text: vec![String::from("null")],
            font_id: 1,
            current_line: 0,

            counter: 0,

            can_continue: false,

            timer: Timer::new(60),
        }
    }

    pub fn update_text(&mut self, pokemon: &String, pmove: &String) {
        // To - do: seperate into two lines not just one
        let mut text = pokemon.clone();
        text.push_str(" used ");
        text.push_str(pmove.as_str());
        text.push('!');
        self.text = vec![text];
    }

    pub fn update_faint(&mut self, pokemon: &String) {
        let mut text = pokemon.clone();
        text.push_str(" fainted!");
        self.text = vec![text];
    }

    fn reset(&mut self) {
        self.counter = 0;
		self.current_line = 0;
        self.can_continue = false;
        self.timer.despawn();
    }

    pub fn update(&mut self) {
        if self.is_active() {
            if !self.can_continue {
                let line_len = self.get_line(self.current_line).len() as u16 * 4;
                if self.counter <= line_len {
                    self.counter += 1;
                } else if self.current_line < self.get_text().len() - 1 {
                    self.current_line += 1;
                    self.counter = 0;
                } else {
                    self.counter = line_len;
                    self.can_continue = true;
                }
            }
        }
    }
}

impl GuiComponent for BattleText {
    fn enable(&mut self) {
        self.alive = true;
        self.reset();
    }

    fn disable(&mut self) {
        self.alive = false;
        self.timer.despawn();
    }

    fn is_active(&self) -> bool {
        self.alive
    }

    fn update(&mut self, _context: &mut GameContext) {
        println!("Wrong update method for battle text");
    }

    fn render(&self, ctx: &mut Context, g: &mut GlGraphics, tr: &mut TextRenderer) {
        if self.is_active() {
            let mut string = String::new();
            let mut count = 0;
            
            for character in self.get_line(self.current_line).chars() {
				if count >= self.counter / 4 {
					break;
				}
				string.push(character);
				count+=1;
			}

			tr.render_text_from_left(ctx, g, self.font_id, string.as_str(), self.panel_x + self.x, self.panel_y + self.y + self.current_line as isize * 16);

			for line_index in 0..self.current_line {
				tr.render_text_from_left(ctx, g, self.font_id, self.get_line(line_index), self.panel_x + self.x, self.panel_y + self.y + line_index as isize * 16);
			}         
        }
    }

    fn update_position(&mut self, x: isize, y: isize) {
        self.panel_x = x;
        self.panel_y = y;
    }
}

impl GuiText for BattleText {
    
    fn get_line(&self, index: usize) -> &String {
        &self.get_text()[index]
    }

    fn get_text(&self) -> &Vec<String> {
        &self.text
    }

    fn get_font_id(&self) -> usize {
        self.font_id
    }
}