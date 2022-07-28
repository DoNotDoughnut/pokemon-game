use std::{ops::Deref, sync::Arc};

use crate::pokedex::trainer::InitTrainer;
use event::EventWriter;

use crate::{
    pokengine::{
        gui::{bag::BagGui, party::PartyGui},
        pokedex::{item::Item, moves::Move, pokemon::Pokemon},
        PokedexClientData,
    },
    state::{MainStates, StateMessage},
};

use crate::engine::{
    controls::{pressed, Control},
    notan::egui,
    utils::Entity,
    App, Plugins,
};

pub struct StartMenu {
    alive: bool,
    cursor: usize,
    party: PartyGui,
    bag: BagGui,
    actions: EventWriter<StateMessage>,
}

impl StartMenu {
    pub(crate) fn new(dex: Arc<PokedexClientData>, sender: EventWriter<StateMessage>) -> Self {
        Self {
            alive: false,
            cursor: 0,
            party: PartyGui::new(dex.clone()),
            bag: BagGui::new(dex),
            actions: sender,
        }
    }

    // #[deprecated]
    // pub fn update(&mut self, app: &mut App, plugins: &mut Plugins, delta: f32) {
    //     if self.bag.alive() {
    //         self.bag.input(app, plugins, &mut user.bag);
    //         // bag_gui.up
    //     } else if self.party.alive() {
    //         // self.party
    //         //     .input(ctx, eng, &self.dex, crate::dex::pokedex(), &mut user.party);
    //     } else {
    //         if pressed(app, plugins, Control::B) || pressed(app, plugins, Control::Start) {
    //             self.despawn();
    //         }

    //         if pressed(app, plugins, Control::A) {
    //             match self.cursor {
    //                 0 => {
    //                     // Save
    //                 }
    //                 1 => {
    //                     // Bag
    //                 }
    //                 2 => {
    //                     // Pokemon
    //                 }
    //                 3 => {
    //                     // Exit to Main Menu

    //                     self.despawn();
    //                 }
    //                 4 => {
    //                     // Close Menu
    //                     self.despawn();
    //                 }
    //                 _ => unreachable!(),
    //             }
    //         }

    //         if pressed(app, plugins, Control::Up) {
    //             if self.cursor > 0 {
    //                 self.cursor -= 1;
    //             } else {
    //                 self.cursor = self.buttons.len() - 1;
    //             }
    //         }
    //         if pressed(app, plugins, Control::Down) {
    //             if self.cursor < self.buttons.len() - 1 {
    //                 self.cursor += 1;
    //             } else {
    //                 self.cursor = 0;
    //             }
    //         }
    //     }
    // }

    pub fn ui(
        &mut self,
        app: &mut App,
        plugins: &mut Plugins,
        egui: &egui::Context,
        user: &mut InitTrainer,
    ) {
        if pressed(app, plugins, Control::Start) {
            self.alive = !self.alive;
        }
        self.bag.ui(egui, &mut user.bag);
        self.party.ui(app, egui, &mut user.party);
        if self.alive {
            egui::Window::new("Menu")
                .title_bar(false)
                .anchor(egui::Align2::RIGHT_TOP, [-5.0, 5.0])
                .show(egui, |ui| {
                    if ui.button("Save").clicked() {
                        self.actions.send(StateMessage::SaveToDisk);
                    }
                    if ui.button("Bag").clicked() {
                        self.bag.spawn();
                    }
                    if ui.button("Party").clicked() {
                        self.party.spawn();
                    }
                    if ui.button("Exit to Menu").clicked() {
                        self.actions.send(StateMessage::Goto(MainStates::Menu));
                    }
                    if ui.button("Close").clicked() {
                        self.alive = false;
                    }
                });
        } else {
            egui::Window::new("Menu Button")
                .title_bar(false)
                .anchor(egui::Align2::RIGHT_TOP, [-5.0, 5.0])
                .show(egui, |ui| {
                    if ui.button("Menu").clicked() {
                        self.alive = true;
                    }
                });
        };
        // if self.alive {
        //     if self.bag.alive() {
        //         self.bag.draw(ctx, eng);
        //     } else if self.party.alive() {
        //         self.party.draw(ctx, eng);
        //     } else {
        //         Panel::draw_text(
        //             ctx,
        //             eng,
        //             self.pos.x,
        //             self.pos.y,
        //             70.0,
        //             &self.buttons,
        //             self.cursor,
        //             false,
        //             false,
        //         );
        //     }
        // }
    }
}

impl Entity for StartMenu {
    fn spawn(&mut self) {
        self.alive = true;
        self.cursor = 0;
    }

    fn despawn(&mut self) {
        self.alive = false;
    }

    fn alive(&self) -> bool {
        self.alive
    }
}
