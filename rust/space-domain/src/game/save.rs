use crate::game::actions::{
    ActionActive, ActionDock, ActionExtract, ActionGeneric, ActionJump, ActionMoveTo,
    ActionRequest, ActionUndock,
};
use crate::game::astrobody::AstroBody;
use crate::game::building_site::BuildingSite;
use crate::game::code::HasCode;
use crate::game::commands::Command;
use crate::game::dock::HasDocking;
use crate::game::events::GEvents;
use crate::game::extractables::Extractable;
use crate::game::factory::Factory;
use crate::game::fleets::Fleet;
use crate::game::label::Label;
use crate::game::locations::{LocationDocked, LocationOrbit, LocationSpace, Moveable};
use crate::game::navigations::{NavRequest, Navigation};
use crate::game::order::TradeOrders;
use crate::game::prefab::Prefab;
use crate::game::production_cost::ProductionCost;
use crate::game::sectors::{Jump, Sector};
use crate::game::shipyard::Shipyard;
use crate::game::station::Station;
use crate::game::utils::{Tick, TotalTime};
use crate::game::wares::{Cargo, Ware};
use bevy_ecs::prelude::*;
use commons::jsons::JsonValueExtra;
use serde::{Deserialize, Serialize};
use space_domain_macros::SaveData;
use std::collections::HashMap;

pub trait LoadingMapEntity {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>);
}

impl<T: LoadingMapEntity> LoadingMapEntity for Option<T> {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        if let Some(value) = self {
            value.map_entity(entity_map);
        }
    }
}

impl<T: LoadingMapEntity> LoadingMapEntity for Vec<T> {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        for value in self {
            value.map_entity(entity_map);
        }
    }
}

impl LoadingMapEntity for Entity {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        *self = entity_map[self];
    }
}

#[derive(Debug, Serialize, Deserialize, SaveData, Default)]
pub struct ObjData {
    pub id: Option<Entity>,
    pub label: Option<Label>,
    pub code: Option<HasCode>,
    pub cargo: Option<Cargo>,
    pub extractable: Option<Extractable>,
    pub location_space: Option<LocationSpace>,
    pub location_docked: Option<LocationDocked>,
    pub fleet: Option<Fleet>,
    pub moveable: Option<Moveable>,
    pub docking: Option<HasDocking>,
    pub station: Option<Station>,
    pub sector: Option<Sector>,
    pub jump_to: Option<Jump>,
    pub command: Option<Command>,
    pub shipyard: Option<Shipyard>,
    pub ware: Option<Ware>,
    pub factory: Option<Factory>,
    pub astro_body: Option<AstroBody>,
    pub location_orbit: Option<LocationOrbit>,
    pub building_site: Option<BuildingSite>,
    pub production_cost: Option<ProductionCost>,
    pub action: Option<ActionActive>,
    pub action_request: Option<ActionRequest>,
    pub action_undock: Option<ActionUndock>,
    pub action_dock: Option<ActionDock>,
    pub action_extract: Option<ActionExtract>,
    pub action_move_to: Option<ActionMoveTo>,
    pub action_jump: Option<ActionJump>,
    pub action_generic: Option<ActionGeneric>,
    pub navigation: Option<Navigation>,
    pub navigation_request: Option<NavRequest>,
    pub trade_order: Option<TradeOrders>,
    pub prefab: Option<Prefab>,
}

impl LoadingMapEntity for ObjData {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        self.cargo.map_entity(entity_map);
        self.extractable.map_entity(entity_map);
        self.location_space.map_entity(entity_map);
        self.location_docked.map_entity(entity_map);
        self.docking.map_entity(entity_map);
        self.sector.map_entity(entity_map);
        self.jump_to.map_entity(entity_map);
        self.command.map_entity(entity_map);
        self.shipyard.map_entity(entity_map);
        self.factory.map_entity(entity_map);
        self.location_orbit.map_entity(entity_map);
        self.building_site.map_entity(entity_map);
        self.production_cost.map_entity(entity_map);
        self.action.map_entity(entity_map);
        self.action_request.map_entity(entity_map);
        self.navigation.map_entity(entity_map);
        self.navigation_request.map_entity(entity_map);
        self.trade_order.map_entity(entity_map);
        self.prefab.map_entity(entity_map);
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SaveData {
    pub tick: Tick,
    pub total_time: TotalTime,
    pub events: GEvents,
    pub objects: Vec<ObjData>,
}

impl LoadingMapEntity for SaveData {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        self.events.map_entity(entity_map);
        self.objects.map_entity(entity_map);
    }
}

pub fn save_world(world: &mut World) -> String {
    log::trace!("saving world");

    let mut save_data = SaveData::default();
    save_data.tick = *world.resource::<Tick>();
    save_data.total_time = *world.resource::<TotalTime>();
    save_data.events = world.resource::<GEvents>().clone();

    for e in world.query::<Entity>().iter(world) {
        let mut obj_data = ObjData::default();
        obj_data.id = Some(e);

        let entity = world.get_entity(e).unwrap();
        obj_data.load_from(&entity);

        log::trace!("saving {:?} {:?}", e, obj_data);

        save_data.objects.push(obj_data);
    }

    let mut ast = serde_json::to_value(&save_data).unwrap();
    ast.strip_nulls();

    let value = serde_json::to_string_pretty(&ast).unwrap();
    log::trace!("save complete, data size of {:?}", value.len());
    value
}

pub fn load_world(world: &mut World, save_data: String) {
    log::trace!("loading world data");
    let mut data: SaveData = serde_json::from_str(&save_data).unwrap();

    // allocate entities
    log::trace!("creating entities and mapping ids");
    let mut entity_map = HashMap::new();

    // sort input objects to try keep similar ids
    data.objects
        .sort_by(|a, b| a.id.unwrap().cmp(&b.id.unwrap()));

    // map id into new entities
    log::trace!("spawning entities");
    for row in &data.objects {
        let id = world.spawn_empty().id();
        log::trace!("created id {:?} for {:?}", id, row.id);
        entity_map.insert(row.id.unwrap(), id);
    }

    // map entities
    log::trace!("mapping entities");
    data.map_entity(&entity_map);

    // insert resources
    log::trace!("loading resources");
    world.insert_resource(data.tick);
    world.insert_resource(data.total_time);
    world.insert_resource(data.events);

    // insert objects
    log::trace!("loading components");
    for row in data.objects {
        let id = entity_map[&row.id.unwrap()];

        log::trace!("loading {:?} {:?}", id, row);
        let mut entity = world.entity_mut(id);
        row.write_into(&mut entity);
    }

    log::trace!("loading complete");
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::actions::Action;
    use crate::game::utils::V2;
    use crate::test::assert_v2;
    use bevy_ecs::entity::Entity;
    use bevy_ecs::prelude::World;
    use log::LevelFilter;

    #[test]
    fn test_save_and_load() {
        _ = env_logger::builder()
            .filter_level(LevelFilter::Trace)
            .try_init();

        let mut world = World::new();
        world.insert_resource(TotalTime(33.0));
        world.insert_resource(GEvents::default());

        let sector_id = world.spawn_empty().id();
        let obj_id = world
            .spawn((
                LocationSpace {
                    pos: V2::new(3.1, 4.5),
                    sector_id: sector_id,
                },
                ActionActive(Action::Undock),
                ActionUndock::default(),
            ))
            .id();

        log::trace!("saving");
        let save_data = save_world(&mut world);

        log::trace!("save data:");
        log::trace!("{}", save_data);

        log::trace!("loading");
        world = World::new();
        load_world(&mut world, save_data);

        log::trace!("result");
        for (obj_id, location, action, action_undock) in world
            .query::<(
                Entity,
                Option<&LocationSpace>,
                Option<&ActionActive>,
                Option<&ActionUndock>,
            )>()
            .iter(&world)
        {
            if obj_id.index() == 0 {
                assert!(location.is_none());
                assert!(action.is_none());
                assert!(action_undock.is_none());
            } else if obj_id.index() == 1 {
                assert_v2(V2::new(3.1, 4.5), location.unwrap().pos);
                assert_eq!(0, location.unwrap().sector_id.index());
                assert_eq!(Action::Undock, action.unwrap().0);
                assert!(action_undock.is_some());
            } else {
                panic!("not expected {:?}", obj_id);
            }
        }
    }
}
