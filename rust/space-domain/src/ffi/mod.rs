use crate::game::loader::{Loader, RandomMapCfg};

#[allow(dead_code)]
use crate::game::Game;
use crate::space_outputs_generated::space_data;
use crate::utils::{DeltaTime, V2};
use flatbuffers::FlatBufferBuilder;

use specs::prelude::*;

use crate::ffi::ffi_output_system::FfiOutputSystem;
use std::time::Duration;

mod ffi_output_system;

pub struct FFI;

pub struct FFIApi {
    game: Game,
    ffi_dispatcher: Dispatcher<'static, 'static>,
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
impl FFIApi {
    pub fn new() -> Self {
        let mut game = Game::new();

        let mut ffi_dispatcher: Dispatcher = DispatcherBuilder::new()
            .with(FfiOutputSystem, "ffi", &[])
            .with_pool(game.thread_pool.clone())
            .build();

        ffi_dispatcher.setup(&mut game.world);

        FFIApi {
            game: game,
            ffi_dispatcher,
        }
    }

    pub fn new_game(&mut self) {
        // Loader::load_advanced_scenery(&mut self.game.world);
        // Loader::load_basic_scenery(&mut self.game);
        Loader::load_random(
            &mut self.game,
            &RandomMapCfg {
                size: 50,
                seed: 50,
                ships: 100,
            },
        );
    }

    pub fn update(&mut self, elapsed: Duration) {
        let delta = DeltaTime((elapsed.as_millis() as f32) / 1000.0);
        self.game.tick(delta);
        self.ffi_dispatcher.run_now(&mut self.game.world);
    }

    pub fn set_inputs(&mut self, bytes: &[u8]) -> bool {
        let inputs = crate::space_inputs_generated::space_data::get_root_as_inputs(bytes);
        if inputs.new_game() {
            self.new_game();
        }
        true
    }

    pub fn get_inputs<F>(&mut self, callback: F) -> bool
    where
        F: FnOnce(Vec<u8>),
    {
        if !self.game.world.has_value::<FfiOutpusBuilder>() {
            log::warn!("fail to get ffi outputs, no FfiOutpusBuilder");
            return false;
        }

        let sectors_to_add = vec![];
        let jumps_to_add = vec![];

        let outputs = &mut self.game.world.write_resource::<FfiOutpusBuilder>();

        for sector in sectors_to_add {
            outputs.sectors_new.push(sector);
        }

        for jump in jumps_to_add {
            outputs.jumps_new.push(jump);
        }

        // TODO: remove copy
        let bytes = outputs.build();
        callback(bytes);

        // clean up outputs
        outputs.clear();

        true
    }
}

pub struct FfiOutpusBuilder {
    pub entities_new: Vec<space_data::EntityNew>,
    pub entities_teleport: Vec<space_data::EntityTeleport>,
    pub entities_moved: Vec<space_data::EntityMove>,
    pub entities_jumped: Vec<space_data::EntityJump>,
    pub entities_dock: Vec<space_data::EntityDock>,
    pub entities_undock: Vec<space_data::EntityUndock>,
    pub sectors_new: Vec<space_data::SectorNew>,
    pub jumps_new: Vec<space_data::JumpNew>,
}

impl Default for FfiOutpusBuilder {
    fn default() -> Self {
        FfiOutpusBuilder::new()
    }
}

impl FfiOutpusBuilder {
    pub fn new() -> Self {
        FfiOutpusBuilder {
            entities_new: vec![],
            entities_teleport: vec![],
            entities_moved: vec![],
            entities_jumped: vec![],
            entities_dock: vec![],
            entities_undock: vec![],
            sectors_new: vec![],
            jumps_new: vec![],
        }
    }

    fn clear(&mut self) {
        self.entities_new = vec![];
        self.entities_moved = vec![];
        self.entities_jumped = vec![];
        self.sectors_new = vec![];
        self.jumps_new = vec![];
    }

    pub fn push_entity_new_in_space(
        &mut self,
        id: u32,
        kind: space_data::EntityKind,
        pos: space_data::V2,
        sector_id: u32,
    ) {
        self.entities_new.push(space_data::EntityNew::new(id, kind));
        self.entities_teleport
            .push(space_data::EntityTeleport::new(id, &pos, sector_id));
    }

    pub fn push_entity_new_docked(
        &mut self,
        id: u32,
        kind: space_data::EntityKind,
        docked_id: u32,
    ) {
        self.entities_new.push(space_data::EntityNew::new(id, kind));
        self.push_entity_dock(id, docked_id);
    }

    pub fn push_entity_dock(&mut self, id: u32, docked_id: u32) {
        self.entities_dock
            .push(space_data::EntityDock::new(id, docked_id));
    }

    pub fn push_entity_undock(&mut self, id: u32, pos: space_data::V2, sector_id: u32) {
        self.entities_undock
            .push(space_data::EntityUndock::new(id, &pos, sector_id));
    }

    pub fn push_entity_move(&mut self, id: u32, pos: space_data::V2) {
        self.entities_moved
            .push(space_data::EntityMove::new(id, &pos));
    }

    pub fn push_entity_jump(&mut self, id: u32, pos: space_data::V2, sector_id: u32) {
        self.entities_jumped
            .push(space_data::EntityJump::new(id, sector_id, &pos));
    }

    // TODO: how to return just reference? To keep the builder we need 'b that I don't
    //       know how to reference from a System Data
    pub fn build(&mut self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::new();

        macro_rules! create_vector {
            ($field:expr) => {
                if $field.is_empty() {
                    None
                } else {
                    let v = std::mem::replace(&mut $field, vec![]);
                    Some(builder.create_vector(v.as_ref()))
                }
            };
        }

        let root_args = space_data::OutputsArgs {
            entities_new: create_vector!(self.entities_new),
            entities_teleport: create_vector!(self.entities_teleport),
            entities_move: create_vector!(self.entities_moved),
            entities_jump: create_vector!(self.entities_jumped),
            entities_dock: create_vector!(self.entities_dock),
            entities_undock: create_vector!(self.entities_undock),
            sectors: create_vector!(self.sectors_new),
            jumps: create_vector!(self.jumps_new),
        };

        let root = space_data::Outputs::create(&mut builder, &root_args);
        builder.finish_minimal(root);

        let bytes = builder.finished_data().into();
        self.clear();
        bytes
    }
}

#[cfg(test)]
mod test {
    use crate::ffi::FfiOutpusBuilder;

    use crate::space_outputs_generated::space_data;

    #[test]
    fn test_events_to_flatoutputs_empty() {
        let mut builder = FfiOutpusBuilder::new();
        let bytes = builder.build();
        let root = space_data::get_root_as_outputs(bytes.as_ref());
        assert!(root.entities_new().is_none());
        assert!(root.entities_jump().is_none());
        assert!(root.entities_move().is_none());
    }

    #[test]
    fn test_events_to_flatoutputs_objects_added() {
        let mut builder = FfiOutpusBuilder::new();
        builder.push_entity_new_docked(0, space_data::EntityKind::Fleet, 1);
        builder.push_entity_new_in_space(
            1,
            space_data::EntityKind::Station,
            space_data::V2::new(2.0, 5.0),
            2,
        );
        let bytes = builder.build();

        let root = space_data::get_root_as_outputs(bytes.as_ref());
        assert!(root.entities_jump().is_none());
        assert!(root.entities_move().is_none());
        match root.entities_new() {
            Some(new_entities) => {
                assert_eq!(new_entities.len(), 2);

                assert_eq!(new_entities[0].id(), 0);
                assert_eq!(new_entities[0].kind(), space_data::EntityKind::Fleet);

                assert_eq!(new_entities[1].id(), 1);
                assert_eq!(new_entities[1].kind(), space_data::EntityKind::Station);
            }
            None => {
                panic!();
            }
        }

        match root.entities_teleport() {
            Some(teleports) => {
                assert_eq!(teleports.len(), 1);

                assert_eq!(teleports[0].id(), 1);
                assert_eq!(teleports[0].pos().x(), 2.0);
                assert_eq!(teleports[0].pos().y(), 5.0);
                assert_eq!(teleports[0].sector_id(), 2);
            }
            None => {
                panic!();
            }
        }

        match root.entities_dock() {
            Some(docked) => {
                assert_eq!(docked.len(), 1);

                assert_eq!(docked[0].id(), 0);
                assert_eq!(docked[0].target_id(), 1);
            }
            None => {
                panic!();
            }
        }
    }

    #[test]
    fn test_events_to_flatoutputs_entities_moved() {
        let mut builder = FfiOutpusBuilder::new();
        builder.push_entity_move(0, space_data::V2::new(22.0, 35.0));
        builder.push_entity_move(1, space_data::V2::new(-1.0, 0.0));
        let bytes = builder.build();

        let root = space_data::get_root_as_outputs(bytes.as_ref());
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
        let mut builder = FfiOutpusBuilder::new();
        builder.push_entity_jump(0, space_data::V2::new(22.0, 35.0), 3);
        builder.push_entity_jump(1, space_data::V2::new(-1.0, 0.0), 4);
        let bytes = builder.build();

        let root = space_data::get_root_as_outputs(bytes.as_ref());
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
