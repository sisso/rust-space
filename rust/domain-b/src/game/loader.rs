use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

use bevy_ecs::prelude::*;
use bevy_ecs::system::{RunSystemOnce, SystemState};
use bevy_utils::WorldExt;

use commons;
use commons::math;
use commons::math::{Distance, Rad, P2, P2I};

use crate::game::astrobody::{AstroBody, AstroBodyKind};
use crate::game::bevy_utils::CommandSendEvent;
use crate::game::building_site::BuildingSite;
use crate::game::code::{Code, HasCode};
use crate::game::commands::Command;
use crate::game::dock::HasDocking;
use crate::game::events::{EventKind, GEvent, GEvents};
use crate::game::extractables::Extractable;
use crate::game::factory::{Factory, Receipt};
use crate::game::fleets::Fleet;
use crate::game::label::Label;
use crate::game::locations::{LocationDocked, LocationOrbit, LocationSpace, Moveable};
use crate::game::new_obj::NewObj;
use crate::game::objects::ObjId;
use crate::game::orbit::Orbits;
use crate::game::order::{TradeOrders, TRADE_ORDER_ID_BUILDING_SITE, TRADE_ORDER_ID_FACTORY};
use crate::game::prefab::{Prefab, PrefabId};
use crate::game::sectors::{Jump, JumpId, Sector, SectorId};
use crate::game::shipyard::{ProductionOrder, Shipyard};
use crate::game::station::Station;
use crate::game::utils::{DeltaTime, Speed, TotalTime, V2};
use crate::game::wares::{CargoDistributionDirty, Ware, WareAmount, WareId, WaresByCode};
use crate::game::{bevy_utils, conf, prefab};

/// AKA commands editor
pub struct Loader {}

impl Loader {
    pub const DEFAULT_ORBIT_SPEED: Speed = Speed(5.0);

    pub fn add_asteroid(
        commands: &mut Commands,
        sector_id: SectorId,
        pos: V2,
        ware_id: WareId,
    ) -> ObjId {
        let asteroid = Self::new_asteroid(sector_id)
            .with_label("asteroid".to_string())
            .with_pos(pos)
            .extractable(Extractable {
                ware_id,
                accessibility: 10.0,
            });
        Loader::add_object(commands, &asteroid)
    }

    pub fn add_shipyard(commands: &mut Commands, sector_id: SectorId, pos: V2) -> ObjId {
        let new_obj = Self::new_station()
            .at_position(sector_id, pos)
            .with_label("shipyard".to_string())
            .with_cargo_size(500)
            .with_shipyard(Shipyard::new());

        Loader::add_object(commands, &new_obj)
    }

    pub fn add_mothership(
        commands: &mut Commands,
        sector_id: SectorId,
        pos: V2,
        receipt: Receipt,
    ) -> ObjId {
        let new_obj = Self::new_station()
            .at_position(sector_id, pos)
            .with_label("mothership")
            .with_cargo_size(500)
            .with_shipyard(Shipyard::new())
            .with_factory(Factory::new(receipt));

        Loader::add_object(commands, &new_obj)
    }

    pub fn add_factory(
        commands: &mut Commands,
        sector_id: SectorId,
        pos: V2,
        receipt: Receipt,
    ) -> ObjId {
        Loader::add_object(commands, &Self::new_factory(sector_id, pos, receipt))
    }

    pub fn new_station() -> NewObj {
        NewObj::new()
            .with_label("station".to_string())
            .with_cargo_size(100)
            .with_station()
            .with_docking()
    }

    pub fn new_factory(sector_id: SectorId, pos: V2, receipt: Receipt) -> NewObj {
        Loader::new_station()
            .at_position(sector_id, pos)
            .with_label(format!("factory {}", receipt.label))
            .with_factory(Factory::new(receipt))
    }

    pub fn add_ship_miner(
        commands: &mut Commands,
        docked_at: ObjId,
        speed: f32,
        label: String,
    ) -> ObjId {
        Loader::add_object(
            commands,
            &Loader::new_ship(speed, label)
                .at_dock(docked_at)
                .with_command(Command::mine()),
        )
    }

    pub fn set_shipyard_order_to_random(
        world: &mut World,
        shipyard_id: ObjId,
    ) -> Result<(), &'static str> {
        Ok(world
            .get_entity_mut(shipyard_id)
            .ok_or("shipyard_id not found")?
            .get_mut::<Shipyard>()
            .ok_or("shipyard has no shipyard")?
            .set_production_order(ProductionOrder::Random))
    }

    pub fn add_ship_trader(
        commands: &mut Commands,
        docked_at: ObjId,
        speed: f32,
        label: String,
    ) -> ObjId {
        Loader::add_object(
            commands,
            &Loader::new_ship(speed, label)
                .at_dock(docked_at)
                .with_command(Command::trade()),
        )
    }

    pub fn new_ship(speed: f32, label: String) -> NewObj {
        NewObj::new()
            .with_cargo_size(20)
            .with_speed(Speed(speed))
            .can_dock()
            .with_label(label)
            .with_fleet()
    }

    // pub fn new_ship2(docked_at: ObjId, speed: f32, label: String) -> NewObj {
    //     NewObj::new()
    //         .with_cargo(20)
    //         .with_speed(Speed(speed))
    //         .at_dock(docked_at)
    //         .can_dock()
    //         .with_label(label)
    //         .with_fleet()
    //         .with_command(Command::mine())
    // }

    pub fn add_sector(commands: &mut Commands, pos: P2I, name: String) -> ObjId {
        Loader::add_object(commands, &NewObj::new().with_sector(pos).with_label(name))
    }

    pub fn add_ware<T: Into<String>>(commands: &mut Commands, code: T, name: T) -> WareId {
        Loader::add_object(
            commands,
            &NewObj::new()
                .with_ware()
                .with_code(code.into())
                .with_label(name.into()),
        )
    }

    pub fn add_jump(
        commands: &mut Commands,
        from_sector_id: SectorId,
        from_pos: P2,
        to_sector_id: JumpId,
        to_pos: P2,
    ) -> (ObjId, ObjId) {
        let jump_from_id = commands
            .spawn_empty()
            .insert(Label::from("jump"))
            .insert(Jump {
                target_sector_id: to_sector_id,
                target_pos: to_pos,
            })
            .insert(LocationSpace {
                pos: from_pos,
                sector_id: from_sector_id,
            })
            .id();

        let jump_to_id = commands
            .spawn_empty()
            .insert(Jump {
                target_sector_id: from_sector_id,
                target_pos: from_pos,
            })
            .insert(LocationSpace {
                pos: to_pos,
                sector_id: to_sector_id,
            })
            .id();

        commands.add(CommandSendEvent::from(GEvent::new(
            jump_from_id,
            EventKind::Add,
        )));
        commands.add(CommandSendEvent::from(GEvent::new(
            jump_to_id,
            EventKind::Add,
        )));

        log::debug!(
            "{:?} creating jump from {:?} to {:?}",
            jump_from_id,
            from_sector_id,
            to_sector_id,
        );
        log::debug!(
            "{:?} creating jump from {:?} to {:?}",
            jump_to_id,
            to_sector_id,
            from_sector_id,
        );

        (jump_from_id, jump_to_id)
    }

    pub fn new_asteroid(sector_id: SectorId) -> NewObj {
        NewObj::new()
            .with_asteroid()
            .at_position(sector_id, V2::ZERO)
    }

    pub fn new_star(sector_id: Entity) -> NewObj {
        NewObj::new().at_position(sector_id, P2::ZERO).with_star()
    }

    pub fn new_planet(sector_id: Entity) -> NewObj {
        NewObj::new().at_position(sector_id, P2::ZERO).with_planet()
    }

    pub fn add_object_from_commands(world: &mut World, new_obj: &NewObj) -> ObjId {
        world.run_commands(|mut commands| Self::add_object(&mut commands, new_obj))
    }

    pub fn add_object_from_world(world: &mut World, new_obj: &NewObj) -> ObjId {
        world.run_commands(|mut commands| Self::add_object(&mut commands, new_obj))
    }

    pub fn add_object(commands: &mut Commands, new_obj: &NewObj) -> ObjId {
        let mut builder = commands.spawn_empty();

        // assert consistency
        if new_obj.cargo.is_none() && (new_obj.shipyard.is_some() || new_obj.factory.is_some()) {
            panic!(
                "invalid obj, shipyard or factory without cargo: {:?}",
                new_obj
            );
        }

        if new_obj.can_dock && new_obj.speed.is_none() {
            panic!(
                "fatal {:?}: entity that can dock should be moveable",
                new_obj
            );
        }

        // create new obj
        let mut orders = TradeOrders::default();

        if let Some(code) = new_obj.code.as_ref() {
            builder.insert(HasCode {
                code: code.to_string(),
            });
        }

        if let Some(label) = new_obj.label.as_ref() {
            builder.insert(Label {
                label: label.to_string(),
            });
        }

        if new_obj.docking {
            builder.insert(HasDocking::default());
        }

        if let Some(orbit) = new_obj.location_orbit.as_ref() {
            builder.insert(orbit.clone());
        }

        if let Some(location) = &new_obj.location_space {
            builder.insert(location.clone());
        }

        if let Some(docked) = &new_obj.location_docked {
            builder.insert(docked.clone());
        }

        if let Some(speed) = &new_obj.speed {
            builder.insert(Moveable {
                speed: speed.clone(),
            });
        }

        if let Some(extractable) = &new_obj.extractable {
            builder.insert(extractable.clone());
        }

        if new_obj.station {
            builder.insert(Station {});
        }

        if new_obj.fleet {
            builder.insert(Fleet {});
        }

        if let Some(sector_pos) = &new_obj.sector {
            builder.insert(Sector::new(sector_pos.clone()));
        }

        for (target_sector_id, target_pos) in &new_obj.jump_to {
            builder.insert(Jump {
                target_sector_id: *target_sector_id,
                target_pos: *target_pos,
            });
        }

        for command in &new_obj.command {
            builder.insert(command.clone());
        }

        for shipyard in &new_obj.shipyard {
            builder.insert(shipyard.clone());
        }

        if let Some(cargo) = &new_obj.cargo {
            let cargo = cargo.clone();
            builder.insert(cargo);
            builder.insert(CargoDistributionDirty {});
        }

        for factory in &new_obj.factory {
            builder.insert(factory.clone());
            for wa in &factory.production.input {
                orders.add_request(TRADE_ORDER_ID_FACTORY, wa.ware_id);
            }
            for wa in &factory.production.output {
                orders.add_provider(TRADE_ORDER_ID_FACTORY, wa.ware_id);
            }
        }

        if let Some(_) = new_obj.star {
            builder.insert(AstroBody {
                kind: AstroBodyKind::Star,
            });
        }

        if let Some(_) = new_obj.planet {
            builder.insert(AstroBody {
                kind: AstroBodyKind::Planet,
            });
        }

        if new_obj.ware {
            builder.insert(Ware {});
        }

        if let Some(building_site) = &new_obj.building_site {
            builder.insert(building_site.clone());
            for ware_id in &building_site.input {
                orders.add_request(TRADE_ORDER_ID_BUILDING_SITE, ware_id.ware_id);
            }
        }

        if let Some(production_cost) = &new_obj.production_cost {
            builder.insert(production_cost.clone());
        }

        if !orders.is_empty() || new_obj.shipyard.is_some() {
            log::debug!("{:?} setting order of {:?}", builder.id(), orders);
            builder.insert(orders);
        }

        let entity = builder.id();

        log::debug!("add_object {:?} from {:?}", entity, new_obj);

        commands.add(CommandSendEvent::from(GEvent::new(entity, EventKind::Add)));

        entity
    }

    pub fn add_prefab(
        commands: &mut Commands,
        code: &str,
        label: &str,
        new_obj: NewObj,
        shipyard: bool,
        building_site: bool,
    ) -> Entity {
        let new_obj_str = format!("{:?}", new_obj);

        let entity = commands
            .spawn_empty()
            .insert(Prefab {
                obj: new_obj,
                shipyard: shipyard,
                build_site: building_site,
            })
            .insert(HasCode::from_str(code))
            .insert(Label::from(label))
            .id();

        log::debug!("add_prefab {:?} from {}", entity, new_obj_str);

        entity
    }

    pub fn new_by_prefab_code(world: &mut World, code: String) -> Option<NewObj> {
        let rs = world.run_system_once_with(code, prefab::find_prefab_by_code);
        rs.map(|p| p.obj.clone())
    }

    pub fn add_by_prefab_code(world: &mut World, code: &str) -> Option<ObjId> {
        let new_obj = Self::new_by_prefab_code(world, code.to_string())?;
        Some(Self::add_object_from_world(world, &new_obj))
    }

    pub fn new_station_building_site(prefab_id: PrefabId, input: Vec<WareAmount>) -> NewObj {
        Self::new_station()
            .with_label("building_site".to_string())
            .with_cargo_size(100)
            .with_building_site(BuildingSite { prefab_id, input })
            .with_docking()
    }

    pub fn set_obj_at_orbit(
        world: &mut World,
        obj_id: ObjId,
        parent_id: ObjId,
        distance: Distance,
        angle: Rad,
        speed: Speed,
    ) {
        let total_time = *world.resource::<TotalTime>();

        world.entity_mut(obj_id).insert(LocationOrbit {
            parent_id,
            distance,
            start_time: total_time,
            start_angle: angle,
            speed,
        });

        Orbits::update_orbits(world);
    }

    pub fn set_obj_to_obj_orbit(world: &mut World, obj_id: ObjId, target_id: ObjId) {
        Loader::set_obj_to_obj_position(world, obj_id, target_id);
        let orbit = LocationOrbit::new(target_id);
        let mut obj = world.get_entity_mut(obj_id).expect("obj_id not found");
        log::debug!(
            "{:?} teleported to target {:?} orbit location on orbit {:?}",
            obj_id,
            target_id,
            orbit,
        );
        obj.insert(orbit);
    }

    pub fn set_obj_to_obj_position(world: &mut World, obj_id: ObjId, target_id: ObjId) {
        let target_location = world
            .get_entity(target_id)
            .expect("target_id not found")
            .get::<LocationSpace>()
            .expect("target has no location")
            .clone();

        let mut obj = world.get_entity_mut(obj_id).expect("obj_id not found");

        log::debug!(
            "{:?} teleported to target {:?} to location {:?}",
            obj_id,
            target_id,
            target_location
        );

        obj.insert(target_location);

        // remove if docked
        obj.remove::<LocationDocked>();
    }

    pub fn set_obj_position(world: &mut World, obj_id: ObjId, location_space: &LocationSpace) {
        log::debug!("{:?} teleported to position {:?}", obj_id, location_space);

        let mut obj = world.get_entity_mut(obj_id).expect("obj_id not found");
        obj.insert(location_space.clone());
        // remove if docked
        obj.remove::<LocationDocked>();
    }

    pub fn set_obj_docked(world: &mut World, obj_id: ObjId, parent_id: ObjId) {
        log::debug!("{:?} teleported docked at {:?}", obj_id, parent_id,);

        let mut entity = world.get_entity_mut(obj_id).expect("obj_id not found");
        entity.remove::<LocationSpace>();
        entity.remove::<LocationOrbit>();
        entity.insert(LocationDocked { parent_id });
    }

    pub fn compute_orbit_speed(radius: Distance) -> Speed {
        let base_speed = Loader::DEFAULT_ORBIT_SPEED.0;
        let speed = math::map_value(radius, 1.0, 10.0, base_speed * 1.5, base_speed * 0.1);
        // log::info!("{:?} radius {:?} speed {:?}", obj_id, radius, speed);
        Speed(speed)
    }
}

pub fn set_orbit_random_body(
    world: &mut World,
    obj_id: ObjId,
    seed: u64,
) -> Result<ObjId, &'static str> {
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut candidates = vec![];
    {
        // get entity sector
        let sector_id = match world
            .get_entity(obj_id)
            .expect("obj_id not found")
            .get::<LocationSpace>()
        {
            None => {
                log::warn!(
                    "obj {:?} it is not in a sector to set a orbit, skipping",
                    obj_id
                );
                return Err("entity is not in sector");
            }
            Some(v) => v.sector_id,
        };

        // find all candidates in sector
        let mut system_state: SystemState<
            Query<(
                Entity,
                &LocationSpace,
                With<AstroBody>,
                Option<&LocationOrbit>,
            )>,
        > = SystemState::new(world);
        let query = system_state.get(world);

        for (i_id, l, _, o) in &query {
            if i_id == obj_id {
                continue;
            }

            if l.sector_id != sector_id {
                continue;
            }

            candidates.push((i_id, o.map(|i| i.distance).unwrap_or(10.0)));
        }

        if candidates.len() == 0 {
            log::warn!(
                "not astro bodies candidates found for sector_id {:?}",
                sector_id
            );
            return Err("no candidate in sector");
        }
    }

    let selected = rng.gen_range(0..candidates.len());
    let base_radius = candidates[selected].1;
    // if base_radius < 0.01 {
    //     base_radius = 10.0;
    // }
    let radius = rng.gen_range((base_radius * 0.1)..(base_radius * 0.5));
    let angle = rng.gen_range(0.0..math::TWO_PI);
    let parent_id = candidates[selected].0;

    let speed = Loader::compute_orbit_speed(radius);
    log::trace!("{:?} radius {:?} speed {:?}", obj_id, radius, speed);
    Loader::set_obj_at_orbit(world, obj_id, parent_id, radius, angle, speed);

    Ok(parent_id)
}

// pub fn load_station_prefab(commands: &mut Commands, station: &conf::Station) -> Option<Entity> {}
fn into_wareamount(wares_by_code: &WaresByCode, code: &str, amount: u32) -> WareAmount {
    let ware_id = wares_by_code
        .get(code)
        .unwrap_or_else(|| panic!("ware {} not found", code));
    WareAmount {
        ware_id,
        amount: amount as u32,
    }
}

fn into_wareamount_list(
    wares_by_code: &WaresByCode,
    list: &[conf::ReceiptWare],
) -> Vec<WareAmount> {
    list.iter()
        .map(|rw| into_wareamount(&wares_by_code, rw.ware.as_str(), rw.amount))
        .collect()
}

pub fn load_prefabs(commands: &mut Commands, prefabs: &conf::Prefabs) {
    // generate wares and collect index
    let mut wares_by_code: HashMap<Code, WareId> = Default::default();
    for ware in &prefabs.wares {
        let ware_id = Loader::add_ware(commands, ware.code.clone(), ware.label.clone());
        wares_by_code.insert(ware.code.clone(), ware_id);
    }
    let wares_by_code = WaresByCode::from(wares_by_code);

    // generate receipts
    let mut receipts: HashMap<String, Receipt> = Default::default();

    for receipt in &prefabs.receipts {
        receipts.insert(
            receipt.code.clone(),
            Receipt {
                label: receipt.label.clone(),
                input: into_wareamount_list(&wares_by_code, &receipt.input),
                output: into_wareamount_list(&wares_by_code, &receipt.output),
                time: DeltaTime(receipt.time),
            },
        );
    }

    // load fleets prefabs
    for fleet in &prefabs.fleets {
        let mut obj = NewObj::new()
            .with_cargo_size(fleet.storage)
            .with_speed(Speed(fleet.speed))
            .with_label(fleet.label.clone());

        if let Some(prod_cost) = fleet.production_cost.as_ref() {
            obj = obj.with_production_cost(
                prod_cost.work,
                into_wareamount_list(&wares_by_code, &prod_cost.cost),
            );
        }

        Loader::add_prefab(commands, &fleet.code, &fleet.label, obj, true, false);
    }

    // create stations prefabs
    for station in &prefabs.stations {
        let mut obj = NewObj::new()
            .with_label(station.label.clone())
            .with_station()
            .with_cargo_size(station.storage as u32)
            .with_docking();

        if let Some(data) = &station.shipyard {
            let mut shipyard = Shipyard::new();
            shipyard.production = data.production;
            obj = obj.with_shipyard(shipyard);
        }

        if let Some(factory) = &station.factory {
            let receipt = receipts
                .get(factory.receipt.as_str())
                .unwrap_or_else(|| panic!("receipt {} not found", factory.receipt))
                .clone();

            obj = obj.with_factory(Factory {
                production: receipt,
                production_time: None,
            });
        }

        if let Some(prod_cost) = station.production_cost.as_ref() {
            obj = obj.with_production_cost(
                prod_cost.work,
                into_wareamount_list(&wares_by_code, &prod_cost.cost),
            );
        }

        Loader::add_prefab(commands, &station.code, &station.label, obj, false, true);
    }
}
