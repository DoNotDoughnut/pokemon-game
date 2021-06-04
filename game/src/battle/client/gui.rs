use std::rc::Rc;

use deps::log::debug;

use crate::{battle::pokemon::{BattleClientMove, BattlePartyTrait, PokemonUnknown}, gui::{bag::BagGui, party::PartyGui}, log::warn, pokedex::{
        item::ItemUseType, 
        moves::target::{
            MoveTarget, 
            MoveTargetInstance, 
            Team
        },
    }, tetra::Context, util::{Entity, Completable, Reset}};

use crate::battle::{
    BattleData, 
    BattleType, 
    pokemon::{
        BattleMove,
        BattlePartyKnown, 
        BattlePartyUnknown,
        BattleClientAction,
        BattleClientActionInstance,
        gui::{ActivePokemonRenderer, ActiveRenderer},
    }, 
    ui::{
        self,
        BattleGui,
        panels::BattlePanels,
    },
};

use super::BattleClient;

pub struct BattlePlayerGui {

    party: Rc<PartyGui>,
    bag: Rc<BagGui>,
	pub gui: BattleGui,

    state: BattlePlayerState,

    is_wild: bool,

    user: BattlePartyKnown,
    pub player_renderer: ActiveRenderer,

    opponent: BattlePartyUnknown,
    pub opponent_renderer: ActiveRenderer,

    moves: Vec<BattleMove>,

    faint: deps::hash::HashMap<usize, usize>,


}

#[derive(Debug)]
struct MoveQueue {
    actions: std::collections::VecDeque<BattleClientActionInstance>,
    current: Option<BattleClientActionInstance>,
}

#[derive(Debug)]
enum BattlePlayerState {
    WaitToSelect,
    Select(usize), // usize = active pokemon num
    // Faint(usize),
    WaitToMove,
    Moving(MoveQueue),
}

// enum MoveState {
//     Start,
// 	SetupPokemon,
// 	Pokemon(Vec<BattleActionInstance>), // queue of pokemon
// 	SetupPost,
// 	Post,
// 	End,
// }

impl Default for BattlePlayerState {
    fn default() -> Self {
        Self::WaitToSelect
    }
}

impl BattlePlayerGui {

    pub fn new(ctx: &mut Context, party: Rc<PartyGui>, bag: Rc<BagGui>) -> Self {
        Self {
            party,
            bag,
			gui: BattleGui::new(ctx),
            state: Default::default(),
            is_wild: false,
            user: BattlePartyKnown::default(),
            player_renderer: Default::default(),
            opponent: Default::default(),
            opponent_renderer: Default::default(),
            moves: Vec::with_capacity(3),
            faint: Default::default(),
        }
    }

    pub fn update(&mut self, ctx: &Context, delta: f32) {
        match &mut self.state {
            BattlePlayerState::WaitToSelect | BattlePlayerState::WaitToMove => (),//debug!("{:?}", self.state),
            BattlePlayerState::Select(active_index) => {
                match self.user.active.get(*active_index) {
                    Some(index) => match index {
                        Some(index) => {
                            let pokemon = &self.user.pokemon[*index];
                            match self.gui.panel.alive {
                                true => match self.moves.len() <= *active_index {
                                    true => {
            
                                        // Checks if a move is queued from an action done in the GUI
            
                                        if self.bag.alive() {
                                            self.bag.input(ctx);
                                            if let Some(item) = self.bag.take_selected_despawn() {
                                                let target = match &item.value().usage {
                                                    ItemUseType::Pokeball => MoveTargetInstance::Opponent(crate::battle::BATTLE_RANDOM.gen_range(0, self.gui.panel.fight.targets.names.len())),
                                                    ItemUseType::Script(..) => todo!("user targeting"),
                                                    ItemUseType::None => todo!("make item unusable"),
                                                    // MoveTarget::Opponents => todo!("make none"),
                                                };
                                                self.moves.push(BattleMove::UseItem(item, target));
                                            }
                                        } else if self.party.alive() {
                                            self.party.input(ctx);
                                            self.party.update(delta);
                                            if let Some(selected) = self.party.take_selected() {
                                                self.party.despawn();
                                                self.moves.push(BattleMove::Switch(selected));
                                            }
                                        } else {
                                            if let Some(panels) = self.gui.panel.input(ctx, pokemon) {
                                                match panels {
                                                    BattlePanels::Main => {
                                                        match self.gui.panel.battle.cursor {
                                                            0 => self.gui.panel.active = BattlePanels::Fight,
                                                            1 => self.bag.spawn(),
                                                            2 => crate::battle::ui::battle_party_known_gui(&self.party, &self.user, true),
                                                            3 => if self.is_wild {
                                                                // closer.spawn(self, &mut gui.text);
                                                                todo!()
                                                                // self.state = BattleState::End; // To - do: "Got away safely!" - run text and conditions
                                                            },
                                                            _ => unreachable!(),
                                                        }
                                                    }
                                                    BattlePanels::Fight => match pokemon.moves.get(self.gui.panel.fight.moves.cursor) {
                                                        Some(instance) => match instance.get() {
                                                            Some(move_ref) => self.moves.push(BattleMove::Move(
                                                                self.gui.panel.fight.moves.cursor,
                                                                match move_ref.value().target {
                                                                    MoveTarget::User => MoveTargetInstance::user(),
                                                                    MoveTarget::Opponent => MoveTargetInstance::opponent(self.gui.panel.fight.targets.cursor),
                                                                    MoveTarget::AllButUser => MoveTargetInstance::all_but_user(*active_index, self.user.active.len()),
                                                                    MoveTarget::Opponents => MoveTargetInstance::opponents(self.opponent.active.len()),
                                                                }
                                                            )),
                                                            None => warn!("Pokemon is out of Power Points for this move!")
                                                        }
                                                        None => warn!("Could not get move at cursor!"),
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    false => {
                                        *active_index += 1;
                                        self.gui.panel.despawn();
                                    }
                                }
                                false => {
                                    
                                    self.gui.panel.user(pokemon);
                                }
                            }
                        },
                        None => *active_index += 1,
                    },
                    None => {
                        self.gui.panel.despawn();
                    },
                }
            },
            // BattlePlayerState::Faint(active) => ,
            BattlePlayerState::Moving(queue) => {

                match &mut queue.current {
                    None => {
                        match queue.actions.pop_front() {
                            Some(instance) => {

                                let (user, user_ui, other, other_ui) = match instance.pokemon.team {
                                    Team::Player => (&mut self.user as &mut dyn BattlePartyTrait, &mut self.player_renderer, &mut self.opponent as &mut dyn BattlePartyTrait, &mut self.opponent_renderer),
                                    Team::Opponent => (&mut self.opponent as _, &mut self.opponent_renderer, &mut self.user as _, &mut self.player_renderer),
                                };

                                self.gui.text.clear();
                                self.gui.text.reset();

                                if user.active(instance.pokemon.index).is_some() || !instance.action.requires_user() {

                                    match &instance.action {
                                        BattleClientAction::Move(pokemon_move, targets) => {

                                            ui::text::on_move(&mut self.gui.text, pokemon_move.value(), user.active(instance.pokemon.index).unwrap());
    
                                            for (target, moves) in targets {

                                                {

                                                    let user = user.active_mut(instance.pokemon.index).unwrap();

                                                    let user_pokemon_ui = &mut user_ui[instance.pokemon.index];

                                                    for moves in moves {
                                                        match moves {
                                                            BattleClientMove::UserHP(damage) => {
                                                                user.set_hp(*damage);
                                                                
                                                            }
                                                            BattleClientMove::Fail => {
                                                                ui::text::on_fail(&mut self.gui.text, vec![format!("{} cannot use move", user.name()), format!("{} (Unimplemented)", pokemon_move.value().name)])
                                                            },
                                                            BattleClientMove::Miss => ui::text::on_miss(&mut self.gui.text, user),
                                                            _ => (),
                                                        }
                                                    }
    
                                                    user_pokemon_ui.update_status(Some(user), false);

                                                }

                                                let (target, target_ui) = match target {
                                                    MoveTargetInstance::Opponent(index) => (other.active_mut(*index).unwrap(), &mut other_ui[*index]),
                                                    MoveTargetInstance::Team(index) => (user.active_mut(*index).unwrap(), &mut user_ui[*index]),
                                                    MoveTargetInstance::User => (user.active_mut(instance.pokemon.index).unwrap(), &mut user_ui[instance.pokemon.index]),
                                                };
        
                                                for moves in moves {
                                                    match moves {
                                                        BattleClientMove::TargetHP(damage) => {
                                                            target.set_hp(*damage);
                                                            if damage >= &0.0 {
                                                                target_ui.renderer.flicker()
                                                            }
                                                        },
                                                        BattleClientMove::Effective(effective) => {
                                                            ui::text::on_effective(&mut self.gui.text, effective);
                                                        },
                                                        BattleClientMove::StatStage(stat, stage) => {
                                                            ui::text::on_stat_stage(&mut self.gui.text, target, *stat, *stage);
                                                        }
                                                        BattleClientMove::Faint(target_instance) => {
                                                            target.set_hp(0.0);
                                                            ui::text::on_faint(&mut self.gui.text, self.is_wild, instance.pokemon.team, target);
                                                            target_ui.renderer.faint();
                                                            queue.actions.push_front(
                                                                BattleClientActionInstance {
                                                                    pokemon: *target_instance,
                                                                    action: BattleClientAction::Faint,
                                                                }
                                                            );
                                                            // exp gain stuff here

                                                        },
                                                        _ => (),
                                                    }
                                                }

                                                target_ui.update_status(Some(target), false);

                                            }
    
                                        }
                                        BattleClientAction::UseItem(item, target) => {
                                            let target = match target {
                                                MoveTargetInstance::Opponent(i) => other.active(*i),
                                                MoveTargetInstance::Team(i) => user.active(*i),
                                                MoveTargetInstance::User => user.active(instance.pokemon.index),
                                            }.unwrap();
                                            ui::text::on_item(&mut self.gui.text, target, item.value())
                                        }
                                        BattleClientAction::Switch(index, unknown_pokemon) => {
                                            if let Some(unknown) = unknown_pokemon {
                                                user.add(*index, *unknown);
                                            }
                                            let coming = user.pokemon(*index).unwrap();
                                            ui::text::on_switch(&mut self.gui.text, user.active(instance.pokemon.index).unwrap(), coming);
                                        }
                                        BattleClientAction::Faint => {
                                            // let target = match instance.pokemon.team {
                                            //     Team::Player => &mut self.user as &mut dyn BattlePartyTrait,
                                            //     Team::Opponent => &mut self.opponent as _,
                                            // };
                                            // ui::text::on_faint(&mut self.gui.text, self.is_wild, instance.pokemon.team, target);
                                            // user_pokemon_ui.renderer.faint();
    
                                            // if instance.pokemon.team == Team::Player {
                                            //     self.start_faint(instance.pokemon.index);
                                            // }
    
                                            // if let Some(assailant) = assailant {
                                            //     if assailant.team == Team::Player {
                                            //         let experience = {
                                            //             let instance = user.active(instance.pokemon.index).unwrap();
                                            //             instance.pokemon().value().exp_from(instance.level()) as f32 * 
                                            //             match self.is_wild {
                                            //                 true => 1.0,
                                            //                 false => 1.5,
                                            //             } *
                                            //             7.0
                                            //         } as crate::pokedex::pokemon::Experience;
                                            //         let (assailant_party, index) = (&mut match assailant.team {
                                            //             Team::Player => &mut self.player,
                                            //             Team::Opponent => &mut self.opponent,
                                            //         }, assailant.active);
                                            //         if let Some(assailant_pokemon) = assailant_party.active[index].pokemon.as_mut() {
                                            //             let level = assailant_pokemon.level;
                                            //             if let Some((level, moves)) = assailant_pokemon.add_exp(experience) {
                                            //                 queue.actions.push_front(BattleActionInstance { pokemon: *assailant, action: BattleAction::LevelUp(level, moves) });
                                            //             }
                                            //             queue.actions.push_front(BattleActionInstance { pokemon: *assailant, action: BattleAction::GainExp(level, experience) });
                                            //         }
                                            //     }
                                            // }
    
                                        },
                                        _ => todo!(),
                                        // BattleClientAction::GainExp(level, experience) => { // To - do: experience spreading
                                        //     ui::text::on_gain_exp(&mut self.gui.text, pokemon, *experience);
                                        //     user_pokemon_ui[instance.pokemon.index].update_status(user.active(instance.pokemon.index), *level, false);
                                        // }
                                        // BattleClientAction::LevelUp(level, moves) => {
                                        //     ui::text::on_level_up(text, pokemon, *level);
                                        //     if let Some(_) = moves {
                                        //         ui::text::on_fail(&mut self.gui.text, vec![format!("To - do: handle moves on level up")]);
                                        //     }
                                        // }
                                        // BattleClientAction::Catch(index) => {
                                        //     if let Some(target) = match index.team {
                                        //         Team::Player => &user.active[index.active],
                                        //         Team::Opponent => &other.active[index.active],
                                        //     }.pokemon.as_ref() {
                                        //         ui::text::on_catch(text, target);
                                        //     }
                                        // }
                                    }

                                    // end of let Some(pokemon)

                                    queue.current = Some(instance);

                                }                                
                            },
                            None => self.state = BattlePlayerState::WaitToSelect,
                        }
                    },
                    Some(instance) => {

                        let (user, user_ui, other, other_ui) = match instance.pokemon.team {
                            Team::Player => (&mut self.user as &mut dyn BattlePartyTrait, &mut self.player_renderer, &mut self.opponent as &mut dyn BattlePartyTrait, &mut self.opponent_renderer),
                            Team::Opponent => (&mut self.opponent as _, &mut self.opponent_renderer, &mut self.user as _, &mut self.player_renderer),
                        };
                        

                        match &mut instance.action {
                            BattleClientAction::Move(_, targets) => {
                                if !self.gui.text.finished() {
                                    self.gui.text.update(ctx, delta);
                                } else if self.gui.text.current > 0 || self.gui.text.can_continue {
                                    let index = instance.pokemon.index;
                                    targets.retain(|(t, _)| {
                                        let ui = match *t {
                                            MoveTargetInstance::Opponent(i) => &other_ui[i],
                                            MoveTargetInstance::Team(i) => &user_ui[i],
                                            MoveTargetInstance::User => &user_ui[index],
                                        };
                                        ui.renderer.flicker.flickering() || ui.status.health_moving()
                                    });
                                    if targets.is_empty() {
                                        queue.current = None;
                                    } else {
                                        for target in targets {
                                            let ui = match target.0 {
                                                MoveTargetInstance::Opponent(i) => &mut other_ui[i],
                                                MoveTargetInstance::Team(i) => &mut user_ui[i],
                                                MoveTargetInstance::User => &mut user_ui[instance.pokemon.index],
                                            };
                                            ui.renderer.flicker.update(delta);
                                            ui.status.update_hp(delta);
                                        }
                                    }                                    
                                }
                            },
                            BattleClientAction::Switch(new, _) => {
                                if self.gui.text.finished() {
                                    queue.current = None;
                                } else {

                                    self.gui.text.update(ctx, delta);

                                    if self.gui.text.current() == 1 {
                                        user.replace(instance.pokemon.index, Some(*new));
                                        user_ui[instance.pokemon.index].update(user.active(instance.pokemon.index));
                                    }

                                }
                            },
                            BattleClientAction::UseItem(_, target) => {
                                let target = match target {
                                    MoveTargetInstance::Opponent(i) => &mut other_ui[*i],
                                    MoveTargetInstance::Team(i) => &mut user_ui[*i],
                                    MoveTargetInstance::User => &mut user_ui[instance.pokemon.index],
                                };
                                if !self.gui.text.finished() {
                                    self.gui.text.update(ctx, delta)
                                } else if target.status.health_moving() {
                                    target.status.update_hp(delta);
                                } else {
                                    queue.current = None;
                                }
                            },
                            BattleClientAction::Faint => {
                                let ui = &mut user_ui[instance.pokemon.index];
                                if ui.renderer.faint.fainting() {
                                	ui.renderer.faint.update(delta);
                                } else if !self.gui.text.finished() {
                                	self.gui.text.update(ctx, delta);
                                } else {
                                    match instance.pokemon.team {
                                        Team::Player => if self.user.any_inactive() {
                                            match self.party.alive() {
                                                true => {
                                                    self.party.input(ctx);
                                                    self.party.update(delta);
                                                    if let Some(selected) = self.party.take_selected() {
                                                        if !self.user.pokemon[selected].fainted() {
                                                            // user.queue_replace(index, selected);
                                                            self.party.despawn();
                                                            self.faint.insert(instance.pokemon.index, selected);
                                                            self.user.replace(instance.pokemon.index, Some(selected));
                                                            ui.update(self.user.active(instance.pokemon.index));
                                                            queue.current = None;
                                                        }
                                                    }
                                                },
                                                false => crate::battle::ui::battle_party_known_gui(&self.party, &self.user, false)
                                            }
                                        } else {
                                            debug!("no inactive!");
                                            self.user.replace(instance.pokemon.index, None);
                                            user_ui[instance.pokemon.index].update(None);
                                            queue.current = None;
                                        },
                                        Team::Opponent => {
                                            debug!("opponent faint!");
                                            queue.current = None;
                                        }
                                    }
                                }
                            }
                            _ => todo!(),
                            // BattleClientAction::Catch(_) => todo!(),
                            // BattleClientAction::GainExp(_, _) => {
                            //     let user = self.user.active_mut(instance.pokemon.index);
                            //     let renderer = user_rend[instance.pokemon.active];
                            //     if !self.gui.text.finished() || cli.status.exp_moving() {
                            //         self.gui.text.update(ctx, delta);
                            //         if self.gui.text.current > 0 || text.can_continue {
                            //             renderer.status.update_exp(delta, user.pokemon.as_ref().unwrap());
                            //         }
                            //     } else {
                            //         queue.current = None;
                            //     }
                            // },
                            // BattleClientAction::LevelUp(_, _) => todo!(),
                        }
                    },
                }
            }
        }
    }

    pub fn on_begin(&mut self, ctx: &mut Context) {
        self.player_renderer = ActivePokemonRenderer::init_known(ctx, &self.user);
        self.opponent_renderer = ActivePokemonRenderer::init_unknown(ctx, &self.opponent);
    }

    pub fn draw(&self, ctx: &mut Context) {
        use crate::{graphics::ZERO, tetra::{math::Vec2, graphics::Color}};
        self.gui.background.draw(ctx, 0.0);
        for active in self.opponent_renderer.iter() {
            active.renderer.draw(ctx, ZERO, Color::WHITE);
            active.status.draw(ctx, 0.0, 0.0);
        }
        match &self.state {
            BattlePlayerState::Select(index) => {
                if self.party.alive() {
                    self.party.draw(ctx);
                } else if self.bag.alive() {
                    self.bag.draw(ctx);
                } else {
                    for (current, active) in self.player_renderer.iter().enumerate() {
                        if &current == index {
                            active.renderer.draw(ctx, Vec2::new(0.0, self.gui.bounce.offset), Color::WHITE);
                            active.status.draw(ctx, 0.0, -self.gui.bounce.offset);
                        } else {
                            active.renderer.draw(ctx, ZERO, Color::WHITE);
                            active.status.draw(ctx, 0.0, 0.0);
                        }
                    }
                    self.gui.draw_panel(ctx);
                    self.gui.panel.draw(ctx);
                }
            },
            // BattlePlayerState::Faint(..) => if self.party.alive() {
            //     self.party.draw(ctx)
            // },
            BattlePlayerState::WaitToSelect | BattlePlayerState::WaitToMove | BattlePlayerState::Moving(..) => {
                for active in self.player_renderer.iter().chain(self.opponent_renderer.iter()) {
                    active.renderer.draw(ctx, ZERO, Color::WHITE);
                    active.status.draw(ctx, 0.0, 0.0);
                }
                self.gui.draw_panel(ctx);
                self.gui.text.draw(ctx);
                if self.party.alive() {
                    self.party.draw(ctx)
                }
            },
        }
        
    }
}

impl BattleClient for BattlePlayerGui {

    fn begin(&mut self, data: &BattleData, user: BattlePartyKnown, targets: BattlePartyUnknown) {
        self.gui.panel.target(&targets);
        self.user = user;
        self.opponent = targets;
        self.is_wild = data.battle_type == BattleType::Wild;
    }

    fn start_select(&mut self) {
        self.state = BattlePlayerState::Select(0);
        self.gui.panel.despawn();
    }

    fn wait_select(&mut self) -> Option<Vec<BattleMove>> {
        match &self.state {
            BattlePlayerState::Select(index) => (index >= &self.user.active.len()).then(|| {
                self.state = BattlePlayerState::WaitToMove;
                self.moves.drain(0..self.moves.len()).collect()
            }),
            _ => None,
        }
    }

    fn start_moves(&mut self, queue: Vec<BattleClientActionInstance>) {
        self.state = BattlePlayerState::Moving(MoveQueue {
            actions: queue.into(),
            current: None,
        });
        self.gui.text.clear();
        self.gui.text.spawn();
    }

    fn wait_faint(&mut self, active: usize) -> Option<usize> {
        self.faint.remove(&active)
    }

    fn opponent_faint_replace(&mut self, active: usize, new: Option<usize>, unknown: Option<PokemonUnknown>) {
        if let (Some(new), Some(unknown)) = (new, unknown) {
            self.opponent.add(new, unknown);
        }
        self.opponent.replace(active, new);
        self.opponent_renderer[active].update(self.opponent.active(active));
    }

    fn wait_finish_turn(&mut self) -> bool {
        match &self.state {
            BattlePlayerState::WaitToSelect => true,
            _ => false,
        }
    }

}