use crate::game::events::{Events};
#[allow(dead_code)]
use crate::game::Game;
use crate::space_outputs_generated::space_data;
use crate::utils::{DeltaTime, Seconds, TotalTime, V2};
use flatbuffers::FlatBufferBuilder;
use std::time::Duration;
use crate::game::loader::Loader;

pub struct GameFFI<'a, 'b> {
    game: Game<'a, 'b>,
    first_outputs: bool,
}

impl From<V2> for space_data::V2 {
    fn from(v2: V2) -> Self {
        space_data::V2::new(v2.x, v2.y)
    }
}

impl From<&V2> for space_data::V2 {
    fn from(v2: &V2) -> Self {
        space_data::V2::new(v2.x, v2.y)
    }
}

/// Represent same interface we intend to use through FFI
impl<'a, 'b> GameFFI<'a, 'b> {
    pub fn new() -> Self {
        GameFFI {
            game: Game::new(),
            first_outputs: true,
        }
    }

    pub fn new_game(&mut self) {
        Loader::load_basic_scenery(&mut self.game);
    }

    pub fn update(&mut self, elapsed: Duration) {
        let delta = DeltaTime((elapsed.as_millis() as f32) / 1000.0);
        self.game.tick(delta)
    }

    pub fn set_inputs(&mut self, bytes: &Vec<u8>) -> bool {
        false
    }

    pub fn get_inputs<F>(&mut self, callback: F) -> bool
    where
        F: FnOnce(Vec<u8>),
    {
        //        info!("game_api", "get_inputs");
        //        let events = self.game.events.take();
        //
        //        let mut builder = OutpusBuilder::new();
        //
        //        if self.first_outputs {
        //            self.first_outputs = false;
        //
        //            for sector_id in self.game.sectors.list() {
        //                builder.sectors_new.push(space_data::SectorNew::new(sector_id.value()));
        //            }
        //
        //            for jump in self.game.sectors.list_jumps() {
        //                builder.jumps_new.push(space_data::JumpNew::new(
        //                    jump.id.0,
        //                    jump.sector_id.0,
        //                    &jump.pos.into(),
        //                    jump.to_sector_id.0,
        //                    &jump.to_pos.into()
        //                ));
        //            }
        //        }
        //
        //        // process events
        //        for event in events {
        //            match event.kind {
        //                EventKind::Add => {
        //                    let kind =
        //                        if self.game.extractables.get_extractable(&event.id).is_some() {
        //                            space_data::EntityKind::Asteroid
        //                        } else if self.game.objects.get(&event.id).has_dock {
        //                            space_data::EntityKind::Station
        //                        } else {
        //                            space_data::EntityKind::Fleet
        //                        };
        //
        //                    match self.game.locations.get_location(&event.id) {
        //                        Some(Location::Space { sector_id, pos} ) => {
        //                            builder.push_entity_new(event.id.value(), pos.into(), sector_id.value(), kind);
        //                        },
        //                        Some(Location::Docked { docked_id } ) => {
        //                            let docked_location = self.game.locations.get_location(docked_id).unwrap().get_space();
        //                            builder.push_entity_new(event.id.value(), docked_location.pos.into(), docked_location.sector_id.value(), kind);
        //                        },
        //                        None => {
        //                            warn!("game_api", "Added {:?}, but has no location", event);
        //                        }
        //                    }
        //                }
        //                EventKind::Jump => {
        //                    let location = self.game.locations.get_location(&event.id).unwrap().get_space();
        //                    builder.push_entity_jump(event.id.value(), location.sector_id.value(), location.pos.into());
        //                },
        //                EventKind::Move => {
        //                    let location = self.game.locations.get_location(&event.id).unwrap().get_space();
        //                    builder.push_entity_move(event.id.value(), location.pos.into());
        //                }
        //            }
        //        }
        //
        //        let bytes = builder.finish();
        //        // TODO: remove copy
        //        callback(Vec::from(bytes));
        //        true
        false
    }
}

struct OutpusBuilder<'a> {
    builder: FlatBufferBuilder<'a>,
    finish: bool,
    pub entities_new: Vec<space_data::EntityNew>,
    pub entities_moved: Vec<space_data::EntityMove>,
    pub entities_jumped: Vec<space_data::EntityJump>,
    pub sectors_new: Vec<space_data::SectorNew>,
    pub jumps_new: Vec<space_data::JumpNew>,
}

impl<'a> OutpusBuilder<'a> {
    pub fn new() -> Self {
        OutpusBuilder {
            builder: FlatBufferBuilder::new(),
            finish: false,
            entities_new: vec![],
            entities_moved: vec![],
            entities_jumped: vec![],
            sectors_new: vec![],
            jumps_new: vec![],
        }
    }

    pub fn push_entity_new(
        &mut self,
        id: u32,
        pos: space_data::V2,
        sector_id: u32,
        kind: space_data::EntityKind,
    ) {
        self.entities_new
            .push(space_data::EntityNew::new(id, &pos, sector_id, kind));
    }

    pub fn push_entity_move(&mut self, id: u32, pos: space_data::V2) {
        self.entities_moved
            .push(space_data::EntityMove::new(id, &pos));
    }

    pub fn push_entity_jump(&mut self, id: u32, sector_id: u32, pos: space_data::V2) {
        self.entities_jumped
            .push(space_data::EntityJump::new(id, sector_id, &pos));
    }

    pub fn finish(&mut self) -> &[u8] {
        macro_rules! create_vector {
            ($field:expr) => {
                if $field.is_empty() {
                    None
                } else {
                    Some(
                        self.builder
                            .create_vector(std::mem::replace(&mut $field, vec![]).as_ref()),
                    )
                }
            };
        }

        if !self.finish {
            self.finish = true;

            let root_args = space_data::OutputsArgs {
                entities_new: create_vector!(self.entities_new),
                entities_move: create_vector!(self.entities_moved),
                entities_jump: create_vector!(self.entities_jumped),
                sectors: create_vector!(self.sectors_new),
                jumps: create_vector!(self.jumps_new),
            };

            let root = space_data::Outputs::create(&mut self.builder, &root_args);
            self.builder.finish_minimal(root);
        }
        self.builder.finished_data()
    }
}

#[cfg(test)]
mod test {
    use crate::game::events::{Events};
    use crate::game::objects::ObjId;
    use crate::game_ffi::OutpusBuilder;
    use crate::space_outputs_generated::space_data;

    #[test]
    fn test_events_to_flatoutputs_empty() {
        let mut builder = OutpusBuilder::new();
        let bytes = builder.finish();
        let root = space_data::get_root_as_outputs(bytes);
        assert!(root.entities_new().is_none());
        assert!(root.entities_jump().is_none());
        assert!(root.entities_move().is_none());
    }

    #[test]
    fn test_events_to_flatoutputs_objects_added() {
        let mut builder = OutpusBuilder::new();
        builder.push_entity_new(
            0,
            space_data::V2::new(22.0, 35.0),
            4,
            space_data::EntityKind::Fleet,
        );
        builder.push_entity_new(
            1,
            space_data::V2::new(2.0, 5.0),
            2,
            space_data::EntityKind::Station,
        );
        let bytes = builder.finish();

        let root = space_data::get_root_as_outputs(bytes);
        assert!(root.entities_jump().is_none());
        assert!(root.entities_move().is_none());
        match root.entities_new() {
            Some(new_entities) => {
                assert_eq!(new_entities.len(), 2);

                assert_eq!(new_entities[0].id(), 0);
                assert_eq!(new_entities[0].pos().x(), 22.0);
                assert_eq!(new_entities[0].pos().y(), 35.0);
                assert_eq!(new_entities[0].sector_id(), 4);
                assert_eq!(new_entities[0].kind(), space_data::EntityKind::Fleet);

                assert_eq!(new_entities[1].id(), 1);
                assert_eq!(new_entities[1].pos().x(), 2.0);
                assert_eq!(new_entities[1].pos().y(), 5.0);
                assert_eq!(new_entities[1].sector_id(), 2);
                assert_eq!(new_entities[1].kind(), space_data::EntityKind::Station);
            }
            None => {
                panic!();
            }
        }
    }

    #[test]
    fn test_events_to_flatoutputs_entities_moved() {
        let mut builder = OutpusBuilder::new();
        builder.push_entity_move(0, space_data::V2::new(22.0, 35.0));
        builder.push_entity_move(1, space_data::V2::new(-1.0, 0.0));
        let bytes = builder.finish();

        let root = space_data::get_root_as_outputs(bytes);
        assert!(root.entities_new().is_none());
        assert!(root.entities_jump().is_none());
        match root.entities_move() {
            Some(entity_moved) => {
                assert_eq!(entity_moved.len(), 2);

                assert_eq!(entity_moved[0].id(), 0);
                assert_eq!(entity_moved[0].pos().x(), 22.0);
                assert_eq!(entity_moved[0].pos().y(), 35.0);

                assert_eq!(entity_moved[1].id(), 1);
                assert_eq!(entity_moved[1].pos().x(), -1.0);
                assert_eq!(entity_moved[1].pos().y(), 0.0);
            }
            None => {
                panic!();
            }
        }
    }

    #[test]
    fn test_events_to_flatoutputs_entities_jumped() {
        let mut builder = OutpusBuilder::new();
        builder.push_entity_jump(0, 3, space_data::V2::new(22.0, 35.0));
        builder.push_entity_jump(1, 4, space_data::V2::new(-1.0, 0.0));
        let bytes = builder.finish();

        let root = space_data::get_root_as_outputs(bytes);
        assert!(root.entities_new().is_none());
        assert!(root.entities_move().is_none());
        match root.entities_jump() {
            Some(e) => {
                assert_eq!(e.len(), 2);

                assert_eq!(e[0].id(), 0);
                assert_eq!(e[0].sector_id(), 3);
                assert_eq!(e[0].pos().x(), 22.0);
                assert_eq!(e[0].pos().y(), 35.0);

                assert_eq!(e[1].id(), 1);
                assert_eq!(e[1].sector_id(), 4);
                assert_eq!(e[1].pos().x(), -1.0);
                assert_eq!(e[1].pos().y(), 0.0);
            }
            None => {
                panic!();
            }
        }
    }
}