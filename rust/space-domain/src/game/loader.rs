use std::collections::HashMap;

use rand::prelude::*;
use specs::prelude::*;

use commons;
use commons::math::{self, P2, P2I};

use crate::game::astrobody::{AstroBodies, AstroBody, AstroBodyKind, OrbitalPos};
use crate::game::building_site::BuildingSite;
use crate::game::code::{Code, HasCode};
use crate::game::commands::Command;
use crate::game::dock::Docking;
use crate::game::events::{Event, EventKind, Events};
use crate::game::extractables::Extractable;
use crate::game::factory::{Factory, Receipt};
use crate::game::fleets::Fleet;
use crate::game::label::Label;
use crate::game::locations::{Location, Moveable};
use crate::game::new_obj::NewObj;
use crate::game::objects::ObjId;
use crate::game::order::{TradeOrders, TRADE_ORDER_ID_BUILDING_SITE, TRADE_ORDER_ID_FACTORY};
use crate::game::prefab::{Prefab, PrefabId};
use crate::game::sectors::{Jump, JumpId, Sector, SectorId};
use crate::game::shipyard::Shipyard;
use crate::game::station::Station;
use crate::game::wares::{Cargo, Ware, WareAmount, WareId, WaresByCode};
use crate::game::{conf, prefab};
use crate::specs_extras::*;
use crate::utils::{DeltaTime, Speed, V2};

pub struct Loader {}

pub struct BasicScenery {
    pub asteroid_id: ObjId,
    pub shipyard_id: ObjId,
    pub miner_id: ObjId,
    pub trader_id: ObjId,
    pub ware_ore_id: WareId,
    pub ware_components_id: WareId,
    pub sector_0: SectorId,
    pub sector_1: SectorId,
    pub component_factory_id: ObjId,
}

impl Loader {
    pub fn add_asteroid(world: &mut World, sector_id: SectorId, pos: V2, ware_id: WareId) -> ObjId {
        let asteroid = Self::new_asteroid(sector_id)
            .with_label("asteroid".to_string())
            .with_pos(pos)
            .extractable(Extractable { ware_id });
        Loader::add_object(world, &asteroid)
    }

    pub fn add_shipyard(world: &mut World, sector_id: SectorId, pos: V2) -> ObjId {
        let new_obj = Self::new_station()
            .at_position(sector_id, pos)
            .with_label("shipyard".to_string())
            .with_cargo_size(500)
            .with_shipyard(Shipyard::new());

        Loader::add_object(world, &new_obj)
    }

    pub fn add_mothership(
        world: &mut World,
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

        Loader::add_object(world, &new_obj)
    }

    pub fn add_factory(world: &mut World, sector_id: SectorId, pos: V2, receipt: Receipt) -> ObjId {
        Loader::add_object(world, &Self::new_factory(sector_id, pos, receipt))
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

    pub fn add_ship_miner(world: &mut World, docked_at: ObjId, speed: f32, label: String) -> ObjId {
        Loader::add_object(
            world,
            &Loader::new_ship(speed, label)
                .at_dock(docked_at)
                .with_command(Command::mine()),
        )
    }

    pub fn add_ship_trader(
        world: &mut World,
        docked_at: ObjId,
        speed: f32,
        label: String,
    ) -> ObjId {
        Loader::add_object(
            world,
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

    pub fn add_sector(world: &mut World, pos: P2I, name: String) -> ObjId {
        Loader::add_object(world, &NewObj::new().with_sector(pos).with_label(name))
    }

    pub fn add_ware<T: Into<String>>(world: &mut World, code: T, name: T) -> WareId {
        Loader::add_object(
            world,
            &NewObj::new()
                .with_ware()
                .with_code(code.into())
                .with_label(name.into()),
        )
    }

    pub fn add_jump(
        world: &mut World,
        from_sector_id: SectorId,
        from_pos: P2,
        to_sector_id: JumpId,
        to_pos: P2,
    ) -> (ObjId, ObjId) {
        let jump_from_id = world
            .create_entity()
            .with(Jump {
                target_sector_id: to_sector_id,
                target_pos: to_pos,
            })
            .with(Location::Space {
                pos: from_pos,
                sector_id: from_sector_id,
            })
            .build();

        let jump_to_id = world
            .create_entity()
            .with(Jump {
                target_sector_id: from_sector_id,
                target_pos: from_pos,
            })
            .with(Location::Space {
                pos: to_pos,
                sector_id: to_sector_id,
            })
            .build();

        let events = &mut world.write_resource::<Events>();
        events.push(Event::new(jump_from_id, EventKind::Add));
        events.push(Event::new(jump_to_id, EventKind::Add));

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

    pub fn add_object(world: &mut World, new_obj: &NewObj) -> ObjId {
        let mut builder = world.create_entity();

        let mut force_trade_order_on_empty = false;
        let mut orders = TradeOrders::default();

        let mut update_docking: Option<ObjId> = None;

        if new_obj.can_dock && new_obj.speed.is_none() {
            panic!(
                "fatal {:?}: entity that can dock should be moveable",
                new_obj
            );
        }

        if let Some(code) = new_obj.code.as_ref() {
            builder.set(HasCode {
                code: code.to_string(),
            })
        }

        if let Some(label) = new_obj.label.as_ref() {
            builder.set(Label {
                label: label.to_string(),
            })
        }

        if new_obj.docking {
            builder.set(Docking::default());
        }

        if let Some(location) = &new_obj.location {
            builder.set(location.clone());

            match location {
                Location::Dock { docked_id } => {
                    update_docking = Some(*docked_id);
                }
                _ => {}
            }
        }

        if let Some(speed) = &new_obj.speed {
            builder.set(Moveable {
                speed: speed.clone(),
            });
        }

        if let Some(extractable) = &new_obj.extractable {
            builder.set(extractable.clone());
        }

        if new_obj.station {
            builder.set(Station {});
        }

        if new_obj.fleet {
            builder.set(Fleet {});
        }

        if let Some(sector_pos) = &new_obj.sector {
            builder.set(Sector::new(sector_pos.clone()));
        }

        for (target_sector_id, target_pos) in &new_obj.jump_to {
            builder.set(Jump {
                target_sector_id: *target_sector_id,
                target_pos: *target_pos,
            });
        }

        for command in &new_obj.command {
            builder.set(command.clone());
        }

        for shipyard in &new_obj.shipyard {
            builder.set(shipyard.clone());
            force_trade_order_on_empty = true;
        }

        if let Some(cargo) = &new_obj.cargo {
            let mut cargo = cargo.clone();
            if let Some(factory) = &new_obj.factory {
                factory.setup_cargo(&mut cargo);
            }
            builder.set(cargo);
        }

        if new_obj.cargo_size > 0 {
            let mut cargo = Cargo::new(new_obj.cargo_size);
            if let Some(factory) = &new_obj.factory {
                factory.setup_cargo(&mut cargo);
            }
            builder.set(cargo);
        }

        for factory in &new_obj.factory {
            builder.set(factory.clone());
            for wa in &factory.production.input {
                log::info!("adding request order for ware {:?}", wa);
                orders.add_request(TRADE_ORDER_ID_FACTORY, wa.ware_id);
            }
            for wa in &factory.production.output {
                orders.add_provider(TRADE_ORDER_ID_FACTORY, wa.ware_id);
            }
        }

        if let Some(_) = new_obj.star {
            builder.set(AstroBody {
                kind: AstroBodyKind::Star,
            });
        }

        if let Some(_) = new_obj.planet {
            builder.set(AstroBody {
                kind: AstroBodyKind::Planet,
            });
        }

        if let Some(orbit) = new_obj.orbit.as_ref() {
            builder.set(OrbitalPos {
                parent: orbit.parent,
                distance: orbit.distance,
                initial_angle: orbit.angle,
            });
        }

        if new_obj.ware {
            builder.set(Ware {});
        }

        if let Some(building_site) = &new_obj.building_site {
            builder.set(building_site.clone());
            for ware_id in &building_site.input {
                orders.add_request(TRADE_ORDER_ID_BUILDING_SITE, ware_id.ware_id);
            }
        }

        if let Some(production_cost) = &new_obj.production_cost {
            builder.set(production_cost.clone());
        }

        if !orders.is_empty() || force_trade_order_on_empty {
            log::debug!("{:?} setting order of {:?}", builder.entity, orders);
            builder.set(orders);
        }

        let entity = builder.build();

        if let Some(docked_id) = update_docking {
            world
                .write_storage::<Docking>()
                .get_mut(docked_id)
                .unwrap()
                .docked
                .push(entity);
        }

        log::debug!("add_object {:?} from {:?}", entity, new_obj);

        let events = &mut world.write_resource::<Events>();
        events.push(Event::new(entity, EventKind::Add));

        entity
    }

    pub fn add_prefab(
        world: &mut World,
        code: &str,
        label: &str,
        new_obj: NewObj,
        shipyard: bool,
        building_site: bool,
    ) -> Entity {
        let new_obj_str = format!("{:?}", new_obj);

        let entity = world
            .create_entity()
            .with(Prefab {
                obj: new_obj,
                shipyard: shipyard,
                build_site: building_site,
            })
            .with(HasCode::from_str(code))
            .with(Label::from(label))
            .build();

        log::debug!("add_prefab {:?} from {}", entity, new_obj_str);

        entity
    }

    pub fn new_by_prefab_code(world: &mut World, code: &str) -> Option<NewObj> {
        prefab::find_prefab_by_code(world, code).map(|p| p.obj)
    }

    pub fn add_by_prefab_code(world: &mut World, code: &str) -> Option<ObjId> {
        let new_obj = Self::new_by_prefab_code(world, code)?;
        Some(Self::add_object(world, &new_obj))
    }

    pub fn new_station_building_site(prefab_id: PrefabId, input: Vec<WareAmount>) -> NewObj {
        Self::new_station()
            .with_label("building_site".to_string())
            .with_cargo_size(100)
            .with_building_site(BuildingSite { prefab_id, input })
            .with_docking()
    }
}

pub fn set_orbit_random_body(world: &mut World, obj_id: ObjId, seed: u64) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut candidates = vec![];

    {
        let entities = world.entities();
        let locations = world.read_storage::<Location>();
        let astros = world.read_storage::<AstroBody>();
        let orbits = world.read_storage::<OrbitalPos>();

        let sector_id = match locations.get(obj_id).and_then(|i| i.as_space()) {
            None => {
                log::warn!("obj {:?} it is not in a sector to set a orbit", obj_id);
                return;
            }
            Some(v) => v.sector_id,
        };

        for (id, l, o, _) in (&entities, &locations, &orbits, &astros).join() {
            if l.get_sector_id() != Some(sector_id) {
                continue;
            }

            candidates.push((id, o.distance));
        }

        if candidates.len() == 0 {
            log::warn!(
                "not astro bodies candidates found for sector_id {:?}",
                sector_id
            );
            return;
        }
    }

    let selected = rng.gen_range(0..candidates.len());
    let mut base_radius = candidates[selected].1;
    // fix stars with radius 0
    if base_radius < 0.01 {
        base_radius = 10.0;
    }
    let radius = rng.gen_range((base_radius * 0.1)..(base_radius * 0.5));
    let angle = rng.gen_range(0.0..math::TWO_PI);
    AstroBodies::set_orbit(world, obj_id, candidates[selected].0, radius, angle);
}

// pub fn load_station_prefab(world: &mut World, station: &conf::Station) -> Option<Entity> {}
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

pub fn load_prefabs(world: &mut World, prefabs: &conf::Prefabs) {
    // generate wares and collect index
    let mut wares_by_code: HashMap<Code, WareId> = Default::default();
    for ware in &prefabs.wares {
        let ware_id = Loader::add_ware(world, ware.code.clone(), ware.label.clone());
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
            .with_label(fleet.label.clone())
            .with_ai();

        if let Some(prod_cost) = fleet.production_cost.as_ref() {
            obj = obj.with_production_cost(
                prod_cost.work,
                into_wareamount_list(&wares_by_code, &prod_cost.cost),
            );
        }

        Loader::add_prefab(world, &fleet.code, &fleet.label, obj, true, false);
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

        Loader::add_prefab(world, &station.code, &station.label, obj, false, true);
    }
}
