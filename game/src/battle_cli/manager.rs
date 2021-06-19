use std::rc::Rc;

use crate::{
	storage::{data_mut, player::PlayerSave},
	gui::{
		party::PartyGui,
		bag::BagGui,
	},
	pokedex::{
		pokemon::instance::BorrowedPokemon,
		trainer::TrainerId,
	},
	input::{debug_pressed, DebugBind},
	graphics::ZERO,
	tetra::{
		Context,
		graphics::Color,
	},
	battle_cli::BattleEntry,
	is_debug,
};

use battle::pokemon::BattlePlayer;

use crate::battle_cli::{
	GameBattle,
	clients::gui::{
		guiref::BattlePlayerGuiRef,
		transition::TransitionState,
	},
	ui::transitions::managers::{
		transition::BattleScreenTransitionManager,
		closer::BattleCloserManager,
	},
};

pub struct BattleManager {

	state: BattleManagerState,
	
	battle: GameBattle,
	
	transition: BattleScreenTransitionManager,
	closer: BattleCloserManager,

	player: BattlePlayerGuiRef,

	pub finished: bool,
	
}

#[derive(Debug)]
pub enum BattleManagerState {
	Begin,
	Transition,
	Battle,
	Closer(TrainerId),
}

impl Default for BattleManagerState {
    fn default() -> Self {
        Self::Begin
    }
}

impl BattleManager {
	
	pub fn new(ctx: &mut Context, party: Rc<PartyGui>, bag: Rc<BagGui>) -> BattleManager {
		
		BattleManager {

			state: BattleManagerState::default(),

			battle: GameBattle::new(crate::pokedex::moves::usage::script::engine()),

			transition: BattleScreenTransitionManager::new(ctx),
			closer: BattleCloserManager::default(),

			player: BattlePlayerGuiRef::new(ctx, party, bag),

			finished: false,

		}
		
	}

	pub fn battle(&mut self, entry: BattleEntry) -> bool { // add battle type parameter
		self.finished = false;
		self.state = BattleManagerState::default();
		let data = data_mut();
		let player = &mut data.party;
		(!(
			player.is_empty() || 
			entry.party.is_empty() ||
			// Checks if player has any pokemon in party that aren't fainted (temporary)
			!player.iter().any(|pokemon| !pokemon.fainted())
		)).then(|| {
				let data = data_mut();
				self.battle.battle(
					BattlePlayer::new(
						data.id,
						None,
						data.party.iter_mut().map(|instance| BorrowedPokemon::Borrowed(instance)).collect(), 
						Box::new(self.player.clone()),
						entry.size
					),
					entry
				)
			}
		);
		self.battle.battle.is_some()
	}

	pub fn update(&mut self, ctx: &mut Context, delta: f32, input_lock: bool) {
		if is_debug() {
			if debug_pressed(ctx, DebugBind::F1) { // exit shortcut
				self.end();
				return;
			}
		}
		if self.battle.battle.is_some() {
			match self.state {
				BattleManagerState::Begin => {
					self.player.get().gui.reset();
					self.state = BattleManagerState::Transition;
					self.transition.state = TransitionState::Begin;

					self.battle.battle.begin();

					self.player.get().on_begin(ctx);

					self.update(ctx, delta, input_lock);
				},
				BattleManagerState::Transition => match self.transition.state {
					TransitionState::Begin => {
						self.transition.begin(ctx, self.player.get().battle_data.type_, &self.battle.trainer);
						self.update(ctx, delta, input_lock);
					},
					TransitionState::Run => self.transition.update(ctx, delta),
					TransitionState::End => {
						self.transition.end();
						self.state = BattleManagerState::Battle;
						self.player.get().start(true);
						self.update(ctx, delta, input_lock);
					}
				}
				BattleManagerState::Battle => {

					let player = self.player.get();

					player.update(ctx, delta, input_lock);

					if player.battling() {
						player.gui.bounce.update(delta);

						self.battle.battle.update();
	
						if let Some(winner) = player.winner() {
							self.state = BattleManagerState::Closer(winner);
						}
					}
				},
				BattleManagerState::Closer(winner) => match self.closer.state {
					TransitionState::Begin => {
						self.closer.begin(self.player.get().battle_data.type_, Some(&winner), self.player.get().opponent.party.trainer.as_ref(), self.battle.trainer.as_ref(), &mut self.player.get().gui.text);
						self.update(ctx, delta, input_lock);
					}
					TransitionState::Run => self.closer.update(ctx, delta, &mut self.player.get().gui.text),
					TransitionState::End => {
						self.closer.end();
						self.state = BattleManagerState::default();
						self.finished = true;
					}
				}
			}
		}
	}

	pub fn winner(&self) -> Option<TrainerId> {
		self.player.get().winner()
	}

	pub fn update_data(&mut self, winner: &TrainerId, player_save: &mut PlayerSave) -> bool {
		self.battle.update_data(winner, player_save)
	}

	pub fn world_active(&self) -> bool {
		matches!(self.state, BattleManagerState::Transition) || self.closer.world_active()		
	}

	pub fn end(&mut self) {
		self.finished = true;
		match self.state {
			BattleManagerState::Begin => (),
			BattleManagerState::Transition => self.transition.state = TransitionState::Begin,
			BattleManagerState::Battle => self.battle.battle.end(),
			BattleManagerState::Closer(..) => self.closer.state = TransitionState::Begin,
		}
	}

	pub fn draw(&self, ctx: &mut Context) {
        if self.battle.battle.is_some() {
			match self.state {
				BattleManagerState::Begin => (),
			    BattleManagerState::Transition => self.transition.draw(ctx),
			    BattleManagerState::Battle => self.player.get().draw(ctx),
			    BattleManagerState::Closer(..) => {
					if !matches!(self.closer.state, TransitionState::End) {
						if !self.world_active() {
							self.player.get().gui.background.draw(ctx, 0.0);
							self.player.get().gui.draw_panel(ctx);
							self.player.get().draw(ctx);
							for active in self.player.get().player.renderer.iter() {
								active.renderer.draw(ctx, ZERO, Color::WHITE);
							}
							self.closer.draw_battle(ctx);
							self.player.get().gui.text.draw(ctx);
						}
						self.closer.draw(ctx);
					}
				}
			}
		}
    }
	
}