use crate::data::player::save::PlayerSave;
use crate::gui::game::party::PokemonPartyGui;
use firecore_pokedex::moves::MoveCategory;
use firecore_pokedex::moves::PokemonMove;
use firecore_pokedex::pokemon::party::PokemonParty;
use firecore_pokedex::pokemon::texture::PokemonTexture;
use firecore_util::Completable;
use firecore_util::Reset;
use macroquad::prelude::Vec2;
use macroquad::prelude::info;
use firecore_util::Entity;
use self::gui::BattleGui;
use firecore_util::battle::BattleType;
use firecore_pokedex::pokemon::battle::BattlePokemon;
use self::battle_party::BattleParty;
use self::transitions::managers::closer::BattleCloserManager;

pub mod manager;

pub mod battle_party;

pub mod gui;

pub mod transitions;

// #[deprecated(since = "0.4.0", note = "Move to seperate crate")]
pub struct Battle {

	battle_type: BattleType,
	
	pub player: BattleParty,
	pub opponent: BattleParty,

	try_run: bool,
	
}

impl Default for Battle {
    fn default() -> Self {
        Self {
			player: BattleParty::default(),
			opponent: BattleParty::default(),

			try_run: bool::default(),
			battle_type: BattleType::default(),
		}
    }
}

impl Battle {
	
	pub fn new(player_pokemon: &PokemonParty, opponent_pokemon: &PokemonParty, battle_type: BattleType) -> Self {
		let mut player = BattleParty::new(player_pokemon, PokemonTexture::Back, Vec2::new(40.0, 113.0));
		for (index, pokemon) in player.pokemon.iter().enumerate() {
			if pokemon.pokemon.current_hp != 0 {
				player.select_pokemon(index);
				break;
			}
		}
		Self {
			player,
			opponent: BattleParty::new(opponent_pokemon, PokemonTexture::Front, Vec2::new(144.0, 74.0)),
			battle_type,
			..Battle::default()
		}
	}

	pub fn verify(&self) -> bool {
		if self.player.pokemon.is_empty() {
			false
		} else if self.opponent.pokemon.is_empty() {
			false
		} else {
			true
		}
	}

	pub fn update(&mut self, delta: f32, battle_gui: &mut BattleGui, closer_manager: &mut BattleCloserManager, party_gui: &mut PokemonPartyGui) {
		
		if self.try_run {
			if self.battle_type == BattleType::Wild {
				closer_manager.spawn();
			}
		}

		// Test if there is a pokemon being selected in the party gui while it is alive

		if party_gui.is_alive() {
			if let Some(selected) = party_gui.selected.take() {

				party_gui.despawn();

				self.player.select_pokemon(selected as usize);

				battle_gui.update_gui(&self, true);

				battle_gui.player_panel.start();
				
			}
		}

		// Update the battle text
		
		else if battle_gui.battle_text.text.is_alive() {
			if !battle_gui.battle_text.text.is_finished() {

				// Despawn the player button panel

				if battle_gui.player_panel.is_alive() {
					battle_gui.player_panel.despawn();
				}

				// Perform the player's move

				if battle_gui.battle_text.perform_player(self) {

					self.player_move();

					battle_gui.battle_text.on_move(self.opponent.active(), &mut battle_gui.opponent_pokemon_gui);

					// Handle opponent fainting to player's move

					if self.opponent.active().faint() {

						// add exp to player

						let gain = self.exp_gain();
						self.player.active_mut().exp += gain;

						// get the maximum exp a player can have at their level

						let max_exp = {
							let player = self.player.active();
							player.pokemon.training.growth_rate.level_exp(player.level)
						};

						// level the player up if they reach a certain amount of exp (and then subtract the exp by the maximum for the previous level)

						let level = if self.player.active().exp > max_exp {
							self.player.pokemon[self.player.active].pokemon.level += 1;
							self.player.pokemon[self.player.active].pokemon.exp -= max_exp;
							let player = self.player.active();
							// info!("{} levelled up to Lv. {}", &player.pokemon.data.name, player.level);
							Some(player.level)
						} else {
							// info!("{} gained {} exp. {} is needed to level up!", self.player.active().pokemon.data.name, gain, max_exp - self.player.active().exp);
							None
						};

						// add the exp gain and level up text to the battle text

						let player = self.player.active();
						battle_gui.battle_text.player_level_up(player.name(), player.exp, level);

					}

					// make sure the actions do not repeat

					self.player.next_move.queued = false;

				} else

				// Perform the opponent's move

				if battle_gui.battle_text.perform_opponent(self) {

					self.opponent_move();

					// Update the player's health bar and add faint text if the player has fainted

					battle_gui.battle_text.on_move(self.player.active(), &mut battle_gui.player_pokemon_gui);

					// make sure the actions do not repeat

					self.opponent.next_move.queued = false;

				}

				// Update the text (so it scrolls)

				battle_gui.battle_text.text.update(delta);

				// if a pokemon has fainted, remove them from screen gradually using BattlePokemonTextureHandler (bad name)

				if let Some(faint_index) = battle_gui.battle_text.faint_index {
					if battle_gui.battle_text.text.can_continue && battle_gui.battle_text.text.current_message() == faint_index {
						if self.player.active().faint() {

							if !self.player.renderer.is_finished() {
								self.player.renderer.update(delta);
							}

						} else if self.opponent.active().faint() {

							if !self.opponent.renderer.is_finished() {
								self.opponent.renderer.update(delta);
							}

						}
					}
				}
				
			} else {

				// Handle player fainting

				if self.player.active().faint() {

					/*
					*	If the player's active pokemon has fainted, check if the player has whited out,
					*	and if so, end the battle, else spawn the party menu to let the player pick another
					*	pokemon to use in battle.
					*/  

					if self.player.all_fainted() {
						closer_manager.spawn();
					} else {

						party_gui.spawn();
						party_gui.on_battle_start(&self.player);
						
						// Reset the pokemon renderer so it renders pokemon

						self.player.renderer.reset();

					}

				}
				
				// Handle opponent fainting
				
				else if self.opponent.active().faint() {

					// check if all of the opponent's pokemon have fainted, and if so, end the battle, else select a pokemon from the opponent's party
					
					if self.opponent.all_fainted() {
						closer_manager.spawn();
					} else {
						let available: Vec<usize> = self.opponent.pokemon.iter().enumerate()
							.filter(|(_, pkmn)| pkmn.pokemon.current_hp != 0)
							.map(|(index, _)| index)
							.collect();
						self.opponent.select_pokemon(available[macroquad::rand::gen_range(0, available.len())]);

						// Update the opponent's pokemon GUI

						battle_gui.update_gui(self, true);

						// Reset the pokemon renderer so it renders pokemon
	
						self.opponent.renderer.reset();
						
					}

					// Once the text is finished, despawn it

					battle_gui.battle_text.text.despawn(); 
					battle_gui.player_panel.start();

				}
				
				// Handle normal move case (no one faints, all moves were completed)

				else {
					// Once the text is finished, despawn it
					battle_gui.battle_text.text.despawn();
					// Spawn the player panel
					battle_gui.player_panel.start();
				}				
			}
		}

	}
	
	pub fn render_pokemon(&self, y_offset: f32) {
		self.player.renderer.render(self.player.active_texture(), y_offset);
		self.opponent.renderer.render(self.opponent.active_texture(), 0.0);
	}

	pub fn player_move(&mut self) {
		let damage = get_move_damage(&self.player.next_move.pokemon_move, self.player.active(), self.opponent.active());
		let opponent = &mut self.opponent.active_mut();
		if damage >= opponent.current_hp {
			opponent.current_hp = 0;
		} else {
			opponent.current_hp -= damage;
		}
	}

	pub fn opponent_move(&mut self) {
		let damage = get_move_damage(&self.opponent.next_move.pokemon_move, self.opponent.active(), self.player.active());
		let player = self.player.active_mut();
		if damage >= player.current_hp {
			player.current_hp = 0;
		} else {
			player.current_hp -= damage;
		}
	}

	pub fn update_data(&self, player_data: &mut PlayerSave) {
		
		player_data.party.pokemon = self.player.pokemon.iter().map(|pokemon| {
			pokemon.pokemon.to_instance()
		}).collect();
		
	}

	pub fn run(&mut self) {
		self.try_run = true;
	}

	fn exp_gain(&self) -> u32 {
		((self.opponent.active().pokemon.training.base_exp * self.opponent.active().level as u16) as f32 * match self.battle_type {
			BattleType::Wild => 1.0,
			_ => 1.5,
		} / 7.0) as u32
	}
	
}

fn get_move_damage(pmove: &PokemonMove, pokemon: &BattlePokemon, recieving_pokemon: &BattlePokemon) -> u16 {
	if if let Some(accuracy) = pmove.accuracy {
		let hit: u8 = macroquad::rand::gen_range(0, 100);
		let test = hit < accuracy;
		// macroquad::prelude::debug!("{} accuracy: {} < {} = {}",  pmove, hit, accuracy, if test { "Hit! "} else { "Miss!" });
		test
	} else {
		true
	} {
		if let Some(power) = pmove.power {
			let effective = pmove.pokemon_type.unwrap_or_default().effective(recieving_pokemon.pokemon.data.primary_type) as f64 * match recieving_pokemon.pokemon.data.secondary_type {
				Some(ptype) => pmove.pokemon_type.unwrap_or_default().effective(ptype) as f64,
				None => 1.0,
			};
			match pmove.category {
				MoveCategory::Status => return 0,
				MoveCategory::Physical => {
					return ((((2.0 * pokemon.level as f64 / 5.0 + 2.0).floor() * pokemon.base.atk    as f64 * power as f64 / recieving_pokemon.base.def    as f64).floor() / 50.0).floor() * effective) as u16 + 2;
				},
				MoveCategory::Special => {
					return ((((2.0 * pokemon.level as f64 / 5.0 + 2.0).floor() * pokemon.base.sp_atk as f64 * power as f64 / recieving_pokemon.base.sp_def as f64).floor() / 50.0).floor() * effective) as u16 + 2;
				}
			}
		} else {
			return 0;
		}
	} else {
		info!("{} missed!", pokemon);
		return 0;
	}	
}