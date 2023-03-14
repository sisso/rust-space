use commons;
use commons::math::{self, P2};
use rand::prelude::*;
use specs::prelude::*;

use crate::game::astrobody::{AstroBodies, AstroBody, AstroBodyKind, OrbitalPos};
use crate::game::commands::Command;
use crate::game::dock::HasDock;
use crate::game::events::{Event, EventKind, Events};
use crate::game::extractables::Extractable;
use crate::game::factory::{Factory, Receipt};
use crate::game::fleets::Fleet;
use crate::game::locations::{Location, Moveable};
use crate::game::new_obj::NewObj;
use crate::game::objects::ObjId;
use crate::game::order::{Order, Orders};
use crate::game::sectors::{Jump, JumpId, Sector, SectorId};
use crate::game::shipyard::Shipyard;
use crate::game::station::Station;
use crate::game::wares::{Cargo, WareAmount, WareId};
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
        Loader::add_object(
            world,
            &NewObj::new()
                .extractable(Extractable { ware_id })
                .at_position(sector_id, pos),
        )
    }

    pub fn add_shipyard(world: &mut World, sector_id: SectorId, pos: V2, ware_id: WareId) -> ObjId {
        Loader::add_object(
            world,
            &NewObj::new()
                .with_cargo(100.0)
                .at_position(sector_id, pos)
                .as_station()
                .with_shipyard(Shipyard::new(WareAmount(ware_id, 5.0), DeltaTime(5.0)))
                .has_dock(),
        )
    }

    pub fn add_factory(world: &mut World, sector_id: SectorId, pos: V2, receipt: Receipt) -> ObjId {
        Loader::add_object(
            world,
            &NewObj::new()
                .with_cargo(100.0)
                .at_position(sector_id, pos)
                .as_station()
                .with_factory(Factory::new(receipt))
                .has_dock(),
        )
    }

    pub fn add_ship_miner(world: &mut World, docked_at: ObjId, speed: f32, label: String) -> ObjId {
        Loader::add_object(
            world,
            &Loader::add_ship(docked_at, speed, label).with_command(Command::mine()),
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
            &Loader::add_ship(docked_at, speed, label).with_command(Command::trade()),
        )
    }

    pub fn add_ship(docked_at: ObjId, speed: f32, label: String) -> NewObj {
        NewObj::new()
            .with_cargo(2.0)
            .with_speed(Speed(speed))
            .at_dock(docked_at)
            .can_dock()
            .with_label(label)
            .as_fleet()
            .with_command(Command::mine())
    }

    pub fn add_sector(world: &mut World, pos: V2, name: String) -> ObjId {
        Loader::add_object(world, &NewObj::new().with_sector(pos).with_label(name))
    }

    pub fn add_ware(world: &mut World, name: String) -> WareId {
        Loader::add_object(world, &NewObj::new().with_ware().with_label(name))
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
        NewObj::new().as_asteroid().at_position(sector_id, V2::ZERO)
    }

    pub fn new_star(sector_id: Entity) -> NewObj {
        NewObj::new().at_position(sector_id, P2::ZERO).as_star()
    }

    pub fn new_planet(sector_id: Entity) -> NewObj {
        NewObj::new().at_position(sector_id, P2::ZERO).as_planet()
    }

    pub fn add_object(world: &mut World, new_obj: &NewObj) -> ObjId {
        let mut builder = world.create_entity();

        let mut orders = vec![];

        if new_obj.can_dock && new_obj.speed.is_none() {
            panic!(
                "fatal {:?}: entity that can dock should be moveable",
                new_obj
            );
        }

        if new_obj.has_dock {
            builder.set(HasDock);
        }

        for location in &new_obj.location {
            builder.set(location.clone());
        }

        for speed in &new_obj.speed {
            builder.set(Moveable {
                speed: speed.clone(),
            });
        }

        for extractable in &new_obj.extractable {
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
            orders.push(Order::WareRequest {
                wares_id: vec![shipyard.input.get_ware_id()],
            });
        }

        if new_obj.cargo_size > 0.0 {
            let mut cargo = Cargo::new(new_obj.cargo_size);
            // TODO: shipyards?
            for factory in &new_obj.factory {
                factory.setup_cargo(&mut cargo);
            }
            builder.set(cargo);
        }

        for factory in &new_obj.factory {
            builder.set(factory.clone());
            orders.push(Order::WareRequest {
                wares_id: factory.production.request_wares_id(),
            });
            orders.push(Order::WareProvide {
                wares_id: factory.production.provide_wares_id(),
            });
        }

        if !orders.is_empty() {
            builder.set(Orders(orders));
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

        let entity = builder.build();

        log::debug!("add_object {:?} from {:?}", entity, new_obj);

        let events = &mut world.write_resource::<Events>();
        events.push(Event::new(entity, EventKind::Add));

        entity
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
