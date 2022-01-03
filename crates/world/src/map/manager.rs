use hashbrown::HashMap;
use rand::{prelude::IteratorRandom, Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use pokedex::moves::MoveId;

use crate::{
    actions::{WorldAction, WorldActions},
    character::{
        npc::{
            group::{MessageColor, NpcGroup, NpcGroupId},
            trainer::TrainerDisable,
            NpcInteract,
        },
        player::PlayerCharacter,
        Movement,
    },
    events::{InputEvent, Sender},
    map::{object::ObjectId, MovementId, WarpDestination, WorldMap},
    positions::{BoundingBox, Coordinate, Direction, Location, Position},
};

use super::{
    battle::BattleEntry,
    chunk::Connection,
    movement::MapMovementResult,
    wild::{WildChances, WildEntry, WildType},
};

use self::{
    random::WorldRandoms,
    tile::{PaletteTileData, PaletteTileDatas},
};

pub mod tile;

mod random;

pub type Maps = HashMap<Location, WorldMap>;

pub struct WorldMapManager<R: Rng + SeedableRng + Clone> {
    pub data: WorldMapData,
    sender: Sender<WorldAction>,
    randoms: WorldRandoms<R>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorldMapData {
    pub maps: Maps,
    pub tiles: PaletteTileDatas,
    pub npcs: HashMap<NpcGroupId, NpcGroup>,
    pub wild: WildChances,
    pub spawn: (Location, Position),
}

impl<R: Rng + SeedableRng + Clone> WorldMapManager<R> {
    pub fn new(data: WorldMapData, sender: Sender<WorldAction>) -> Self {
        Self {
            data,
            sender,
            randoms: Default::default(),
        }
    }

    pub fn seed(&mut self, seed: u64) {
        self.randoms.seed(seed);
    }

    pub fn contains(&self, location: &Location) -> bool {
        self.data.maps.contains_key(location)
    }

    pub fn get(&self, location: &Location) -> Option<&WorldMap> {
        self.data.maps.get(location)
    }

    pub fn on_warp(&mut self, player: &mut PlayerCharacter) {
        self.on_map_change(player);
        self.on_tile(player);
    }

    pub fn on_map_change(&self, player: &PlayerCharacter) {
        if let Some(map) = self.data.maps.get(&player.location) {
            self.on_change(map);
        }
    }

    pub fn on_change(&self, map: &WorldMap) {
        self.sender.send(WorldActions::PlayMusic(map.music));
    }

    pub fn input(&mut self, player: &mut PlayerCharacter, input: InputEvent) {
        match input {
            InputEvent::Move(direction) => self.try_move(player, direction),
            InputEvent::Interact => self.try_interact(player),
        }
    }

    pub fn try_interact(&mut self, player: &mut PlayerCharacter) {
        if let Some(map) = self.data.maps.get_mut(&player.location) {
            if player.world.npc.active.is_none() {
                let pos = if map
                    .tile(player.position.coords)
                    .map(|tile| {
                        PaletteTileData::iter(&self.data.tiles, &map.palettes)
                            .any(|t| t.forwarding.contains(&tile))
                    })
                    .unwrap_or_default()
                {
                    player.position.in_direction(player.position.direction)
                } else {
                    player.position
                };
                for (id, npc) in map.npcs.iter_mut() {
                    if (npc.interact.is_some() || npc.trainer.is_some()) && npc.interact_from(&pos)
                    {
                        player.world.npc.active = Some(*id);
                        break;
                    }
                }
            }
            let forward = player
                .position
                .coords
                .in_direction(player.position.direction);

            if let Some(object) = map.object_at(&forward) {
                const TREE: &ObjectId = unsafe { &ObjectId::new_unchecked(1701147252) };
                const CUT: &MoveId = unsafe { &MoveId::new_unchecked(7632227) };

                const ROCK: &ObjectId = unsafe { &ObjectId::new_unchecked(1801678706) };
                /// "rock-smash"
                const ROCK_SMASH: &MoveId =
                    unsafe { &MoveId::new_unchecked(493254510180952753532786) };

                fn try_break(
                    sender: &Sender<WorldAction>,
                    location: &Location,
                    coordinate: Coordinate,
                    id: &MoveId,
                    player: &mut PlayerCharacter,
                ) {
                    if player
                        .trainer
                        .party
                        .iter()
                        .any(|p| p.moves.iter().any(|m| &m.0 == id))
                    {
                        sender.send(WorldActions::BreakObject(coordinate));
                        player.world.insert_object(location, coordinate);
                    }
                }

                match &object.group {
                    TREE => try_break(&self.sender, &map.id, forward, CUT, player),
                    ROCK => try_break(&self.sender, &map.id, forward, ROCK_SMASH, player),
                    _ => (),
                }
            }

            if let Some(item) = map.item_at(&forward) {
                player.world.insert_object(&map.id, forward);

                // fix lol

                let bag = &mut player.trainer.bag;

                bag.insert_saved(item.item);
            }
        }
    }

    pub fn post_battle(&mut self, player: &mut PlayerCharacter, winner: bool) {
        player.unfreeze();
        if winner {
            if let Some(entry) = player.world.battle.battling.take() {
                if let Some(trainer) = entry.trainer {
                    if let Some(npc) = self
                        .data
                        .maps
                        .get(&trainer.location)
                        .map(|map| map.npcs.get(&trainer.id).map(|npc| npc.trainer.as_ref()))
                        .flatten()
                        .flatten()
                    {
                        match &npc.disable {
                            TrainerDisable::DisableSelf => {
                                player.world.battle.insert(&trainer.location, trainer.id);
                            }
                            TrainerDisable::Many(others) => {
                                player.world.battle.insert(&trainer.location, trainer.id);
                                player
                                    .world
                                    .battle
                                    .battled
                                    .get_mut(&trainer.location)
                                    .unwrap()
                                    .extend(others);
                            }
                        }
                    }
                }
            }
        } else {
            let (loc, pos) = player.world.heal.unwrap_or(self.data.spawn);
            player.location = loc;
            player.position = pos;
            player.location = player.location;
            player
                .trainer
                .party
                .iter_mut()
                .for_each(|o| o.heal(None, None));
        }
    }

    pub fn move_npcs(&mut self, player: &mut PlayerCharacter, delta: f32) {
        if let Some(map) = self.data.maps.get_mut(&player.location) {
            // Move Npcs

            for npc in player
                .world
                .scripts
                .npcs
                .values_mut()
                .filter(|(location, ..)| map.contains(location))
                .map(|(.., npc)| npc)
            {
                npc.character.do_move(delta);
            }

            for npc in map.npcs.values_mut() {
                npc.character.do_move(delta);
            }

            use crate::character::npc::NpcMovement;

            match player.world.npc.timer > 0.0 {
                false => {
                    player.world.npc.timer += 1.0;

                    const NPC_MOVE_CHANCE: f64 = 1.0 / 12.0;

                    for npc in map.npcs.values_mut() {
                        if !npc.character.moving() && self.randoms.npc.gen_bool(NPC_MOVE_CHANCE) {
                            for movement in npc.movement.iter() {
                                match movement {
                                    NpcMovement::Look(directions) => {
                                        if let Some(direction) =
                                            directions.iter().choose(&mut self.randoms.npc)
                                        {
                                            npc.character.position.direction = *direction;
                                        }
                                        if let Some(trainer) = npc.trainer.as_ref() {
                                            player.find_battle(
                                                &map.id,
                                                &npc.id,
                                                trainer,
                                                &mut npc.character,
                                            );
                                        }
                                    }
                                    NpcMovement::Move(area) => {
                                        let origin =
                                            npc.origin.get_or_insert(npc.character.position.coords);

                                        let next = npc.character.position.forwards();

                                        let bb = BoundingBox::centered(*origin, *area);

                                        if bb.contains(&next) {
                                            if let Some(code) = map.movements.get(
                                                npc.character.position.coords.x as usize
                                                    + npc.character.position.coords.y as usize
                                                        * map.width as usize,
                                            ) {
                                                if WorldMap::can_move(
                                                    npc.character.position.elevation,
                                                    *code,
                                                ) {
                                                    npc.character
                                                        .pathing
                                                        .queue
                                                        .push(npc.character.position.direction);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                true => player.world.npc.timer -= delta,
            }
        }
    }

    pub fn update_interactions(&mut self, player: &mut PlayerCharacter) {
        if let Some(id) = player.world.npc.active.as_ref() {
            let npc = self
                .data
                .maps
                .get_mut(&player.location)
                .map(|map| map.npcs.get(&id))
                .flatten();
            let npc = if let Some(npc) = npc {
                Some(npc)
            } else {
                // player
                //     .world
                //     .scripts
                //     .npcs
                //     .get(id)
                //     .filter(|(location, ..)| &player.location == location)
                //     .map(|(.., npc)| npc)
                None
            };
            if let Some(npc) = npc {
                match &player.world.polling {
                    Some(finished) => {
                        if finished.get() {
                            if let Some(battle) = BattleEntry::trainer(
                                &mut player.world.battle,
                                &player.location,
                                id,
                                npc,
                            ) {
                                self.sender.send(WorldActions::Battle(battle));
                            }
                            if player.frozen() {
                                player.unfreeze();
                            }
                            player.world.npc.active = None;
                            player.world.polling = None;
                        }
                    }
                    None => {
                        if !npc.character.moving() {
                            if !player.world.battle.battled(&player.location, id) {
                                if let Some(group) = self.data.npcs.get(&npc.group) {
                                    if let Some(trainer) = npc.trainer.as_ref() {
                                        // if trainer.battle_on_interact {
                                        // Spawn text window
                                        if let Some(music) = group
                                            .trainer
                                            .as_ref()
                                            .map(|trainer| trainer.music)
                                            .flatten()
                                        {
                                            self.sender.send(WorldActions::PlayMusic(music));
                                        }
                                        player.character.position.direction =
                                            npc.character.position.direction.inverse();
                                        let message = trainer.encounter.iter().map(|lines| lines.iter().map(|line| {
                                                let string = crate::character::message::process_str(line, &npc.character);
                                                crate::character::message::process_str_player(&string, player)
                                            }).collect()).collect();
                                        player.world.polling = self.sender.send_polling(
                                            WorldActions::Message(message, group.message),
                                        );
                                        return;
                                        // }
                                    }
                                }
                            }

                            match &npc.interact {
                                NpcInteract::Message(pages) => {
                                    let message = pages
                                        .iter()
                                        .map(|lines| {
                                            lines
                                                .iter()
                                                .map(|line| {
                                                    let string =
                                                        crate::character::message::process_str(
                                                            line,
                                                            &npc.character,
                                                        );
                                                    crate::character::message::process_str_player(
                                                        &string, player,
                                                    )
                                                })
                                                .collect()
                                        })
                                        .collect();

                                    let color = self
                                        .data
                                        .npcs
                                        .get(&npc.group)
                                        .map(|group| group.message)
                                        .unwrap_or(MessageColor::Black);

                                    player.world.polling = self
                                        .sender
                                        .send_polling(WorldActions::Message(message, color));

                                    return player.position.direction =
                                        npc.character.position.direction.inverse();
                                }
                                NpcInteract::Script(_) => todo!(),
                                NpcInteract::Nothing => (),
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn on_tile(
        &mut self,
        player: &mut PlayerCharacter,
        // party: &[OwnedPokemon<P, M, I>],
    ) {
        self.sender.send(WorldActions::OnTile);

        if let Some(map) = self.data.maps.get_mut(&player.location) {
            if player.world.wild.encounters {
                if let Some(current) = map.tile(player.position.coords) {
                    if PaletteTileData::iter(&self.data.tiles, &map.palettes)
                        .any(|t| t.wild.contains(&current))
                    {
                        let t = match player.movement {
                            Movement::Swimming => &WildType::Water,
                            _ => &WildType::Land,
                        };
                        if let Some(entry) =
                            &map.wild.as_ref().map(|entries| entries.get(t)).flatten()
                        {
                            if let Some(entry) = WildEntry::generate(
                                &self.data.wild,
                                t,
                                entry,
                                &mut self.randoms.wild,
                            ) {
                                self.sender.send(WorldActions::Battle(entry));
                            }
                        }
                    }
                }
            }

            if player.world.npc.active.is_none() {
                for npc in map.npcs.values_mut() {
                    if let Some(trainer) = &npc.trainer {
                        player.find_battle(&map.id, &npc.id, trainer, &mut npc.character);
                    }
                }
            }
        }

        // if let Some(tile_id) = map.tile(player.position.coords) {
        //     // look for player

        //     // try running scripts

        //     if player.world.scripts.actions.is_empty() {
        //         'scripts: for script in map.scripts.iter() {
        //             use worldlib::script::world::Condition;
        //             for condition in &script.conditions {
        //                 match condition {
        //                     Condition::Location(location) => {
        //                         if !location.in_bounds(&player.position.coords) {
        //                             continue 'scripts;
        //                         }
        //                     }
        //                     Condition::Activate(direction) => {
        //                         if player.position.direction.ne(direction) {
        //                             continue 'scripts;
        //                         }
        //                     }
        //                     Condition::NoRepeat => {
        //                         if player.world.scripts.executed.contains(&script.identifier) {
        //                             continue 'scripts;
        //                         }
        //                     }
        //                     Condition::Script(script, happened) => {
        //                         if player.world.scripts.executed.contains(script).ne(happened) {
        //                             continue 'scripts;
        //                         }
        //                     }
        //                     Condition::PlayerHasPokemon(is_true) => {
        //                         if party.is_empty().eq(is_true) {
        //                             continue 'scripts;
        //                         }
        //                     }
        //                 }
        //             }
        //             player
        //                 .world
        //                 .scripts
        //                 .actions
        //                 .extend_from_slice(&script.actions);
        //             player.world.scripts.actions.push(
        //                 worldlib::script::world::WorldAction::Finish(script.identifier),
        //             );
        //             player.world.scripts.actions.reverse();
        //             break;
        //         }
        //     }
        // }
    }

    fn stop_player(&mut self, player: &mut PlayerCharacter) {
        player.stop_move();

        if let Some(map) = self.data.maps.get(&player.location) {
            if let Some(destination) = map.warp_at(&player.position.coords) {
                // Warping does not trigger tile actions!
                player.world.warp = Some(*destination);
            } else if map.in_bounds(player.position.coords) {
                self.on_tile(player);
            }
        }
    }

    pub fn update(&mut self, player: &mut PlayerCharacter, delta: f32) {
        if player.do_move(delta) {
            self.stop_player(player);
        }
        self.move_npcs(player, delta);

        self.update_interactions(player);
    }

    pub fn try_move(&mut self, player: &mut PlayerCharacter, direction: Direction) {
        player.on_try_move(direction);

        let offset = direction.tile_offset();
        let coords = player.position.coords + offset;

        if let Some(map) = self.get(&player.location) {
            // Check for warp on tile
            if player.world.warp.is_none() {
                if let Some(destination) = map.warp_at(&coords) {
                    player.world.warp = Some(*destination);
                    self.sender.send(WorldActions::BeginWarpTransition(coords));
                    return;
                }
            };

            // Check for one-way tile
            if map
                .tile(coords)
                .map(|tile| {
                    PaletteTileData::iter(&self.data.tiles, &map.palettes).any(|t| {
                        t.cliffs
                            .get(&direction)
                            .map(|tiles| tiles.contains(&tile))
                            .unwrap_or_default()
                    })
                })
                .unwrap_or_default()
                && !player.noclip
            {
                self.sender.send(WorldActions::PlayerJump);
                return;
            }

            match map.chunk_movement(coords, player) {
                MapMovementResult::Option(code) => {
                    with_code(player, code.unwrap_or(1), direction);
                }
                MapMovementResult::Chunk(direction, offset, connection) => {
                    if let Some((location, coords, code)) =
                        self.connection_movement(direction, offset, connection, player)
                    {
                        if with_code(player, code, direction) {
                            player.character.position.coords = coords;
                            player.location = location;
                            self.on_map_change(&player);
                        }
                    }
                }
            }
        }

        fn with_code(player: &mut PlayerCharacter, code: MovementId, direction: Direction) -> bool {
            if WorldMap::can_move(player.position.elevation, code) || player.noclip {
                if WorldMap::WATER == code {
                    if player.movement != Movement::Swimming {
                        const SURF: &MoveId = unsafe { &MoveId::new_unchecked(1718777203) };

                        if player
                            .trainer
                            .party
                            .iter()
                            .flat_map(|pokemon| pokemon.moves.iter())
                            .any(|m| &m.0 == SURF)
                            || player.noclip
                        {
                            player.movement = Movement::Swimming;
                        } else {
                            return false;
                        }
                    }
                } else if player.movement == Movement::Swimming {
                    player.movement = Movement::Walking;
                }
                WorldMap::change_elevation(&mut player.position.elevation, code);
                player.pathing.queue.push(direction);
                return true;
                // self.player.offset =
                //     direction.pixel_offset(self.player.speed() * 60.0 * delta);
            }
            false
        }

        // // check if on unwalkable tile
        // let stuck = map.local_movement(player.position.coords)
        //             .map(|code| !can_move(player.movement, code))
        //             .unwrap_or(false);
        // println!("{}", stuck);
    }

    pub fn connection_movement(
        &self,
        direction: Direction,
        offset: i32,
        connections: &[Connection],
        player: &PlayerCharacter,
    ) -> Option<(Location, Coordinate, MovementId)> {
        connections.iter().find_map(|connection| {
            self.data
                .maps
                .get(&connection.0)
                .map(|map| {
                    let o = offset - connection.1;
                    let position = Connection::offset(direction, map, o);
                    let coords = position.in_direction(direction);
                    map.local_movement(coords, player)
                        .map(|code| (map.id, position, code))
                })
                .flatten()
        })
    }

    pub fn warp(&mut self, player: &mut PlayerCharacter, destination: WarpDestination) -> bool {
        match self.data.maps.contains_key(&destination.location) {
            true => {
                player.position.from_destination(destination.position);
                player.pathing.clear();
                player.location = destination.location;
                true
            }
            false => false,
        }
    }
}
