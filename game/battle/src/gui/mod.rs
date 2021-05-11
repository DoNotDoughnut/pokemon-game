use game::{
	util::Entity,
	macroquad::prelude::Vec2,
	graphics::draw,
};

use background::BattleBackground;
use panels::BattlePanel;
use bounce::PlayerBounce;

// use self::panels::level_up::LevelUpMovePanel;

pub mod background;
pub mod panels;
pub mod status;
pub mod text;

pub mod bounce;
pub mod exp_bar;


#[derive(Debug, Clone, Copy)]
pub enum BattleGuiPosition {
	Top, // index and size
	Bottom,
}

#[derive(Debug, Clone, Copy)]
pub struct BattleGuiPositionIndex {
	pub position: BattleGuiPosition,
	pub index: u8,
	pub size: u8,
}

impl BattleGuiPositionIndex {

	pub const fn new(position: BattleGuiPosition, index: u8, size: u8) -> Self {
		Self {
		    position,
		    index,
		    size,
		}
	}

}

pub struct BattleGui {

	pub background: BattleBackground,

	pub panel: BattlePanel,

	pub text: game::gui::text::DynamicText,

	pub bounce: PlayerBounce,

	// pub level_up: LevelUpMovePanel,

}

impl BattleGui {

	pub fn new() -> Self {

		let panel = Vec2::new(0.0, 113.0);

		Self {

			background: BattleBackground::new(),

			panel: BattlePanel::new(),

			text: text::new(),

			bounce: PlayerBounce::new(),

			// level_up: LevelUpMovePanel::new(panel),

		}

	}

	#[inline]
	pub fn render_panel(&self) {
        draw(self.background.panel, 0.0, 113.0);
	}

}

pub fn battle_party_gui(gui: &mut game::gui::party::PartyGui, party: &crate::pokemon::BattleParty, exitable: bool) {
    gui.spawn(party.collect_cloned().into_iter().map(|instance| game::gui::pokemon::PokemonDisplay::new(instance)).collect(), Some(false), exitable);
}