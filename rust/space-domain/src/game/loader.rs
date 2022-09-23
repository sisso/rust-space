use flatbuffers::buffer_has_identifier;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;

use commons;
use rand::prelude::*;
use space_galaxy::system_generator;
use space_galaxy::system_generator::{BodyDesc, UniverseCfg};
use space_galaxy::terrain_generator::Shape::Island;
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
use crate::game::{sectors, Game};
use crate::specs_extras::*;
use crate::utils::{DeltaTime, Position, Speed, V2};

const SECTOR_MAX_SX: i32 = 10;

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

pub struct RandomMapCfg {
    pub size: usize,
    pub seed: u64,
    pub ships: usize,
    pub universe_cfg: space_galaxy::system_generator::UniverseCfg,
}

impl Loader {
    pub fn load_random(game: &mut Game, cfg: &RandomMapCfg) {
        let mut rng: StdRng = SeedableRng::seed_from_u64(cfg.seed);

        let sector_kind_empty = 0;
        let sector_kind_asteroid = 1;
        let sector_kind_power = 2;
        let sector_kind_factory = 3;
        let sector_kind_shipyard = 4;
        let sector_kind_prob = vec![
            commons::prob::Weighted {
                prob: 1.0,
                value: sector_kind_empty,
            },
            commons::prob::Weighted {
                prob: 1.0,
                value: sector_kind_asteroid,
            },
            commons::prob::Weighted {
                prob: 1.0,
                value: sector_kind_factory,
            },
            commons::prob::Weighted {
                prob: 1.0,
                value: sector_kind_power,
            },
            commons::prob::Weighted {
                prob: 0.1,
                value: sector_kind_shipyard,
            },
        ];

        let world = &mut game.world;

        // add configurations
        world.insert(cfg.universe_cfg.clone());

        // wares and receipts
        let ware_ore_id = Loader::add_ware(world, "ore".to_string());
        let ware_components_id = Loader::add_ware(world, "components".to_string());
        let ware_energy = Loader::add_ware(world, "energy".to_string());

        let receipt_process_ores = Receipt {
            input: vec![WareAmount(ware_ore_id, 2.0), WareAmount(ware_energy, 1.0)],
            output: vec![WareAmount(ware_components_id, 1.0)],
            time: DeltaTime(1.0),
        };
        let receipt_produce_energy = Receipt {
            input: vec![],
            output: vec![WareAmount(ware_energy, 1.0)],
            time: DeltaTime(5.0),
        };

        // create sectors
        generate_sectors(world, cfg.size, rng.gen());
        populate_sectors(world, rng.gen());

        // add stations
        {
            fn sector_pos<R: rand::Rng>(rng: &mut R) -> V2 {
                V2::new(
                    (rng.gen_range(0..10) - 5) as f32,
                    (rng.gen_range(0..10) - 5) as f32,
                )
            }

            let mut sectors_id = vec![];
            {
                let entities = world.entities();
                let sectors_repo = world.read_storage::<Sector>();
                for (sector_id, _) in (&entities, &sectors_repo).join() {
                    sectors_id.push(sector_id);
                }
            }

            let mut required_kinds = [false, false, false, false];
            loop {
                for sector_id in &sectors_id {
                    let sector_id = *sector_id;

                    match commons::prob::select_weighted(&mut rng, &sector_kind_prob) {
                        Some(i) if *i == sector_kind_asteroid => {
                            required_kinds[0] = true;
                            Loader::add_asteroid(
                                world,
                                sector_id,
                                sector_pos(&mut rng),
                                ware_ore_id,
                            );
                        }
                        Some(i) if *i == sector_kind_shipyard => {
                            required_kinds[1] = true;

                            Loader::add_shipyard(
                                world,
                                sector_id,
                                sector_pos(&mut rng),
                                ware_components_id,
                            );
                        }
                        Some(i) if *i == sector_kind_factory => {
                            required_kinds[2] = true;

                            Loader::add_factory(
                                world,
                                sector_id,
                                sector_pos(&mut rng),
                                receipt_process_ores.clone(),
                            );
                        }
                        Some(i) if *i == sector_kind_power => {
                            required_kinds[3] = true;

                            Loader::add_factory(
                                world,
                                sector_id,
                                sector_pos(&mut rng),
                                receipt_produce_energy.clone(),
                            );
                        }
                        _ => {}
                    }
                }

                // check if all required stations existrs
                if required_kinds.iter().find(|i| !**i).is_none() {
                    log::warn!(
                        "world generator fail to provide require stations {:?}, retrying",
                        required_kinds
                    );
                    break;
                }
            }
        }

        // add ships
        {
            let mut shipyards = vec![];

            {
                let entities = world.entities();
                let shipyard_storage = world.read_storage::<Shipyard>();
                for (e, _) in (&entities, &shipyard_storage).join() {
                    shipyards.push(e);
                }
            }

            for i in 0..cfg.ships {
                let shipyard = commons::prob::select(&mut rng, &shipyards).unwrap();
                if rng.gen_range(0..=1) == 0 {
                    Loader::add_ship_miner(world, shipyard.to_owned(), 1.0, format!("miner-{}", i));
                } else {
                    Loader::add_ship_trader(
                        world,
                        shipyard.to_owned(),
                        1.0,
                        format!("trader-{}", i),
                    );
                }
            }
        }
    }

    /// Basic scenery used for testing and samples
    ///
    /// Is defined as a simple 2 sector, one miner ship, a station and asteroid
    pub fn load_basic_scenery(game: &mut Game) -> BasicScenery {
        let world = &mut game.world;

        // init wares
        let ware_ore_id = Loader::add_ware(world, "ore".to_string());
        let ware_components_id = Loader::add_ware(world, "components".to_string());

        // init sectors
        let sector_0 = Loader::add_sector(world, V2::new(0.0, 0.0), "Sector 0".to_string());
        let sector_1 = Loader::add_sector(world, V2::new(1.0, 0.0), "Sector 1".to_string());

        Loader::add_jump(
            world,
            sector_0,
            V2::new(0.5, 0.3),
            sector_1,
            V2::new(0.0, 0.0),
        );
        sectors::update_sectors_index(world);

        // init objects
        let asteroid_id = Loader::add_asteroid(world, sector_1, V2::new(-2.0, 3.0), ware_ore_id);
        let component_factory_id = Loader::add_factory(
            world,
            sector_0,
            V2::new(3.0, -1.0),
            Receipt {
                input: vec![WareAmount(ware_ore_id, 2.0)],
                output: vec![WareAmount(ware_components_id, 1.0)],
                time: DeltaTime(1.0),
            },
        );
        let shipyard_id =
            Loader::add_shipyard(world, sector_0, V2::new(1.0, -3.0), ware_components_id);
        let miner_id = Loader::add_ship_miner(world, shipyard_id, 2.0, "miner".to_string());
        let trader_id =
            Loader::add_ship_trader(world, component_factory_id, 2.0, "trader".to_string());

        // return scenery
        BasicScenery {
            asteroid_id,
            shipyard_id,
            miner_id,
            trader_id,
            ware_ore_id,
            ware_components_id,
            sector_0,
            sector_1,
            component_factory_id,
        }
    }

    /// Advanced scenery
    pub fn load_advanced_scenery(world: &mut World) {
        // init wares
        let ware_ore_id = Loader::add_ware(world, "ore".to_string());
        let ware_components_id = Loader::add_ware(world, "components".to_string());
        let ware_energy = Loader::add_ware(world, "energy".to_string());

        // receipts
        let receipt_process_ores = Receipt {
            input: vec![WareAmount(ware_ore_id, 2.0), WareAmount(ware_energy, 1.0)],
            output: vec![WareAmount(ware_components_id, 1.0)],
            time: DeltaTime(1.0),
        };
        let receipt_produce_energy = Receipt {
            input: vec![],
            output: vec![WareAmount(ware_energy, 1.0)],
            time: DeltaTime(5.0),
        };

        // init sectors
        let sector_0 = Loader::add_sector(world, V2::new(0.0, 0.0), "sector 0".to_string());
        let sector_1 = Loader::add_sector(world, V2::new(1.0, 0.0), "sector 1".to_string());

        Loader::add_jump(
            world,
            sector_0,
            V2::new(0.5, 0.3),
            sector_1,
            V2::new(0.0, 0.0),
        );
        sectors::update_sectors_index(world);

        // init objects
        Loader::add_asteroid(world, sector_1, V2::new(-2.0, 3.0), ware_ore_id);
        Loader::add_asteroid(world, sector_1, V2::new(-2.2, 2.8), ware_ore_id);
        Loader::add_asteroid(world, sector_1, V2::new(-2.8, 3.1), ware_ore_id);

        let component_factory_id =
            Loader::add_factory(world, sector_0, V2::new(3.0, -1.0), receipt_process_ores);

        let _energy_factory_id =
            Loader::add_factory(world, sector_0, V2::new(-0.5, 1.5), receipt_produce_energy);

        let shipyard_id =
            Loader::add_shipyard(world, sector_0, V2::new(1.0, -3.0), ware_components_id);
        Loader::add_ship_miner(world, shipyard_id, 2.0, "miner".to_string());
        Loader::add_ship_trader(world, component_factory_id, 2.0, "trader".to_string());
    }

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
        from_pos: Position,
        to_sector_id: JumpId,
        to_pos: Position,
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

    pub fn new_star(
        sector_id: Entity,
        index: usize,
        parent: usize,
        distance: f32,
        angle: f32,
    ) -> NewObj {
        NewObj::new()
            .at_position(sector_id, Position::zero())
            .as_star()
            .with_orbit(index, parent, distance, angle)
    }

    pub fn new_planet(
        sector_id: Entity,
        index: usize,
        parent: usize,
        distance: f32,
        angle: f32,
    ) -> NewObj {
        NewObj::new()
            .at_position(sector_id, Position::zero())
            .as_planet()
            .with_orbit(index, parent, distance, angle)
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
                system_index: orbit.index,
                parent_index: orbit.parent_index,
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

pub fn generate_sectors(world: &mut World, size: usize, seed: u64) {
    let mut sectors_by_index = vec![];

    let galaxy = space_galaxy::galaxy_generator::Galaxy::new(space_galaxy::galaxy_generator::Cfg {
        seed,
        size: size as i32,
    });

    // add sectors
    for s in &galaxy.sectors.list {
        let pos = V2::new(s.coords.x as f32, s.coords.y as f32);
        let sector_id = Loader::add_sector(world, pos, format!("sector {}", s.id));
        sectors_by_index.push((sector_id, pos));
    }
    // add portals
    let mut cached: HashSet<(usize, usize)> = Default::default();

    for j in &galaxy.jumps {
        if !cached.insert((j.sector_a, j.sector_b)) {
            continue;
        }

        if !cached.insert((j.sector_b, j.sector_a)) {
            continue;
        }

        Loader::add_jump(
            world,
            sectors_by_index[j.sector_a].0,
            j.pos_a.into(),
            sectors_by_index[j.sector_b].0,
            j.pos_b.into(),
        );
    }

    sectors::update_sectors_index(world);
}

pub fn populate_sectors(world: &mut World, seed: u64) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let mut sectors_id = vec![];
    {
        let sectors = &world.read_storage::<Sector>();
        let entities = &world.entities();
        for (e, _) in (entities, sectors).join() {
            sectors_id.push(e);
        }
    }

    let universe_cfg = {
        let universe_cfg = world.read_resource::<UniverseCfg>();
        universe_cfg.deref().clone()
    };

    for sector_id in sectors_id {
        let system = system_generator::new_system(&universe_cfg, rng.gen());

        // create bodies
        for body in system.bodies {
            match body.desc {
                BodyDesc::Star { kind } => {
                    let new_obj = Loader::new_star(
                        sector_id,
                        body.index,
                        body.parent,
                        body.distance,
                        body.angle,
                    );
                    Loader::add_object(world, &new_obj);
                }
                BodyDesc::AsteroidField { resources } => {}
                BodyDesc::Planet(planet) => {
                    let new_obj = Loader::new_planet(
                        sector_id,
                        body.index,
                        body.parent,
                        body.distance,
                        body.angle,
                    );
                    Loader::add_object(world, &new_obj);
                }
            };
        }
    }

    AstroBodies::update_orbits(world);
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[test]
    pub fn test_random_scenery() {
        env_logger::builder()
            .filter(None, log::LevelFilter::Debug)
            .try_init();

        let mut game = Game::new();
        let universe_cfg = space_galaxy::system_generator::new_config_from_file(&PathBuf::from(
            "../data/system_generator.conf",
        ));
        Loader::load_random(
            &mut game,
            &RandomMapCfg {
                size: 3,
                seed: 0,
                ships: 3,
                universe_cfg,
            },
        );
    }
}
