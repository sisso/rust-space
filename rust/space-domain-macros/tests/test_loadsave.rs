use bevy_ecs::prelude::*;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use space_domain_macros::SaveData;
use std::collections::HashMap;

pub trait MapEntity {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>);
}

impl<T: MapEntity> MapEntity for Option<T> {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        if let Some(value) = self {
            value.map_entity(entity_map);
        }
    }
}

#[derive(Debug, Serialize, Deserialize, SaveData, Default)]
struct Data {
    id: Option<Entity>,
    c1: Option<C1>,
    c2: Option<C2>,
    c3: Option<C3>,
}

impl MapEntity for Data {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        self.c3.map_entity(entity_map);
    }
}

#[derive(Debug, Component, Serialize, Deserialize, Clone)]
struct C1 {
    name: String,
}

#[derive(Debug, Component, Serialize, Deserialize, Clone)]
enum C2 {
    A,
    B(i32),
    C { value: f32 },
}

#[derive(Debug, Component, Serialize, Deserialize, Clone)]
struct C3 {
    parent: Option<Entity>,
}

impl MapEntity for C3 {
    fn map_entity(&mut self, entity_map: &HashMap<Entity, Entity>) {
        self.parent.iter_mut().for_each(|value| {
            *value = entity_map[value];
        });
    }
}

pub fn save(world: &mut World) -> Vec<u8> {
    let mut table = vec![];

    for e in world.query::<Entity>().iter(world) {
        log::trace!("saving {:?}", e);
        let mut data = Data::default();
        data.id = Some(e);

        let entity = world.get_entity(e).unwrap();
        data.load_from(&entity);
        table.push(data);
    }

    let value = serde_json::to_string(&table).unwrap();
    log::trace!("{}", value);
    value.into_bytes()
}

pub fn load(world: &mut World, data: Vec<u8>) {
    let data_str = String::from_utf8(data).unwrap();
    let table: Vec<Data> = serde_json::from_str(&data_str).unwrap();

    if table.is_empty() {
        return;
    }

    // allocate entities
    let mut entity_map = HashMap::new();

    for row in &table {
        let id = world.spawn_empty().id();
        log::trace!("map {:?} -> {:?}", row.id, id);
        entity_map.insert(row.id.unwrap(), id);
    }

    log::trace!("mapping call");

    for mut row in table {
        let id = entity_map[&row.id.unwrap()];

        // map entities manually
        row.map_entity(&entity_map);

        log::trace!("{:?}", row);
        let mut entity = world.entity_mut(id);
        row.write_into(&mut entity);
    }
}

#[test]
fn test_all() {
    env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .init();

    let mut world = World::new();

    for _ in 0..5 {
        let id = world.spawn_empty().id();
        world.despawn(id);
    }

    let e1 = world
        .spawn((
            C1 {
                name: "one".to_string(),
            },
            C2::B(32),
        ))
        .id();

    world.spawn(());

    world.spawn((
        C1 {
            name: "one".to_string(),
        },
        C3 { parent: Some(e1) },
    ));

    log::trace!("saving");
    let data = save(&mut world);

    log::trace!("loading");
    world = World::new();
    load(&mut world, data);

    log::trace!("result");
    for i in world
        .query::<(Entity, Option<&C1>, Option<&C2>, Option<&C3>)>()
        .iter(&world)
    {
        log::trace!("{:?}", i);
    }
}
