use std::collections::HashSet;

use specs::prelude::*;

use crate::game::commands::Command;
use crate::game::dock::HasDock;
use crate::game::events::{Event, EventKind, Events};
use crate::game::extractables::Extractable;
use crate::game::factory::{Factory, Receipt};
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
}

impl Loader {
    pub fn load_random_map(game: &mut Game, cfg: &RandomMapCfg) {
        use commons::random_grid;
        use rand::prelude::*;
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

        // wares and receipts
        let ware_ore_id = Loader::new_ware(world, "ore".to_string());
        let ware_components_id = Loader::new_ware(world, "components".to_string());
        let ware_energy = Loader::new_ware(world, "energy".to_string());

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
        {
            fn sector_pos<R: rand::Rng>(rng: &mut R) -> V2 {
                V2::new(
                    (rng.gen_range(0..10) - 5) as f32,
                    (rng.gen_range(0..10) - 5) as f32,
                )
            }

            let rgcfg = random_grid::RandomGridCfg {
                width: cfg.size,
                height: cfg.size,
                portal_prob: 0.5,
                deep_levels: 1,
            };

            let grids = random_grid::RandomGrid::new(&rgcfg, &mut rng);
            println!("{:?}", grids);
            let grid = &grids.levels[0];

            let mut sectors_by_index = vec![];

            for i in 0..grid.len() {
                // create sector
                let (x, y) = grid.get_coords(i);
                let pos = V2::new(x as f32, y as f32);
                let sector_id = Loader::new_sector(world, pos, format!("sector {}", i));
                sectors_by_index.push(sector_id);
            }

            // add portals
            {
                let mut cached: HashSet<(usize, usize)> = Default::default();

                for index in 0..grid.len() {
                    for other in grid.neighbors_connected(index) {
                        if !cached.insert((index, other)) {
                            continue;
                        }

                        Loader::new_jump(
                            world,
                            sectors_by_index[index],
                            sector_pos(&mut rng),
                            sectors_by_index[other],
                            sector_pos(&mut rng),
                        );
                    }
                }

                sectors::update_sectors_index(world);
            }

            // add stations
            {
                for sector_id in &sectors_by_index {
                    match commons::prob::select_weighted(&mut rng, &sector_kind_prob) {
                        Some(i) if *i == sector_kind_asteroid => {
                            Loader::new_asteroid(
                                world,
                                *sector_id,
                                sector_pos(&mut rng),
                                ware_ore_id,
                            );
                        }
                        Some(i) if *i == sector_kind_shipyard => {
                            Loader::new_shipyard(
                                world,
                                *sector_id,
                                sector_pos(&mut rng),
                                ware_components_id,
                            );
                        }
                        Some(i) if *i == sector_kind_factory => {
                            Loader::new_factory(
                                world,
                                *sector_id,
                                sector_pos(&mut rng),
                                receipt_process_ores.clone(),
                            );
                        }
                        Some(i) if *i == sector_kind_power => {
                            Loader::new_factory(
                                world,
                                *sector_id,
                                sector_pos(&mut rng),
                                receipt_produce_energy.clone(),
                            );
                        }
                        _ => {}
                    }
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
                    Loader::new_ship_miner(world, shipyard.to_owned(), 1.0, format!("miner-{}", i));
                } else {
                    Loader::new_ship_trader(
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
        let ware_ore_id = Loader::new_ware(world, "ore".to_string());
        let ware_components_id = Loader::new_ware(world, "components".to_string());

        // init sectors
        let sector_0 = Loader::new_sector(world, V2::new(0.0, 0.0), "Sector 0".to_string());
        let sector_1 = Loader::new_sector(world, V2::new(1.0, 0.0), "Sector 1".to_string());

        Loader::new_jump(
            world,
            sector_0,
            V2::new(0.5, 0.3),
            sector_1,
            V2::new(0.0, 0.0),
        );
        sectors::update_sectors_index(world);

        // init objects
        let asteroid_id = Loader::new_asteroid(world, sector_1, V2::new(-2.0, 3.0), ware_ore_id);
        let component_factory_id = Loader::new_factory(
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
            Loader::new_shipyard(world, sector_0, V2::new(1.0, -3.0), ware_components_id);
        let miner_id = Loader::new_ship_miner(world, shipyard_id, 2.0, "miner".to_string());
        let trader_id =
            Loader::new_ship_trader(world, component_factory_id, 2.0, "trader".to_string());

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
        let ware_ore_id = Loader::new_ware(world, "ore".to_string());
        let ware_components_id = Loader::new_ware(world, "components".to_string());
        let ware_energy = Loader::new_ware(world, "energy".to_string());

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
        let sector_0 = Loader::new_sector(world, V2::new(0.0, 0.0), "sector 0".to_string());
        let sector_1 = Loader::new_sector(world, V2::new(1.0, 0.0), "sector 1".to_string());

        Loader::new_jump(
            world,
            sector_0,
            V2::new(0.5, 0.3),
            sector_1,
            V2::new(0.0, 0.0),
        );
        sectors::update_sectors_index(world);

        // init objects
        Loader::new_asteroid(world, sector_1, V2::new(-2.0, 3.0), ware_ore_id);
        Loader::new_asteroid(world, sector_1, V2::new(-2.2, 2.8), ware_ore_id);
        Loader::new_asteroid(world, sector_1, V2::new(-2.8, 3.1), ware_ore_id);

        let component_factory_id =
            Loader::new_factory(world, sector_0, V2::new(3.0, -1.0), receipt_process_ores);

        let _energy_factory_id =
            Loader::new_factory(world, sector_0, V2::new(-0.5, 1.5), receipt_produce_energy);

        let shipyard_id =
            Loader::new_shipyard(world, sector_0, V2::new(1.0, -3.0), ware_components_id);
        Loader::new_ship_miner(world, shipyard_id, 2.0, "miner".to_string());
        Loader::new_ship_trader(world, component_factory_id, 2.0, "trader".to_string());
    }

    pub fn new_asteroid(world: &mut World, sector_id: SectorId, pos: V2, ware_id: WareId) -> ObjId {
        Loader::add_object(
            world,
            NewObj::new()
                .extractable(Extractable { ware_id })
                .at_position(sector_id, pos),
        )
    }

    pub fn new_shipyard(world: &mut World, sector_id: SectorId, pos: V2, ware_id: WareId) -> ObjId {
        Loader::add_object(
            world,
            NewObj::new()
                .with_cargo(100.0)
                .at_position(sector_id, pos)
                .as_station()
                .with_shipyard(Shipyard::new(WareAmount(ware_id, 5.0), DeltaTime(5.0)))
                .has_dock(),
        )
    }

    pub fn new_factory(world: &mut World, sector_id: SectorId, pos: V2, receipt: Receipt) -> ObjId {
        Loader::add_object(
            world,
            NewObj::new()
                .with_cargo(100.0)
                .at_position(sector_id, pos)
                .as_station()
                .with_factory(Factory::new(receipt))
                .has_dock(),
        )
    }

    pub fn new_ship_miner(world: &mut World, docked_at: ObjId, speed: f32, label: String) -> ObjId {
        Loader::add_object(
            world,
            Loader::new_ship(docked_at, speed, label).with_command(Command::mine()),
        )
    }

    pub fn new_ship_trader(
        world: &mut World,
        docked_at: ObjId,
        speed: f32,
        label: String,
    ) -> ObjId {
        Loader::add_object(
            world,
            Loader::new_ship(docked_at, speed, label).with_command(Command::trade()),
        )
    }

    pub fn new_ship(docked_at: ObjId, speed: f32, label: String) -> NewObj {
        NewObj::new()
            .with_cargo(2.0)
            .with_speed(Speed(speed))
            .at_dock(docked_at)
            .can_dock()
            .with_label(label)
            .with_command(Command::mine())
    }

    pub fn new_sector(world: &mut World, pos: V2, name: String) -> ObjId {
        Loader::add_object(world, NewObj::new().with_sector(pos).with_label(name))
    }

    pub fn new_ware(world: &mut World, name: String) -> WareId {
        Loader::add_object(world, NewObj::new().with_ware().with_label(name))
    }

    pub fn new_jump(
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

        info!(
            "{:?} creating jump from {:?} to {:?}",
            jump_from_id, from_sector_id, to_sector_id
        );
        info!(
            "{:?} creating jump from {:?} to {:?}",
            jump_to_id, to_sector_id, from_sector_id
        );

        (jump_from_id, jump_to_id)
    }

    // TODO: receive new obj or reference?
    pub fn add_object(world: &mut World, new_obj: NewObj) -> ObjId {
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

        let entity = builder.build();

        info!("add_object {:?} from {:?}", entity, new_obj);

        let events = &mut world.write_resource::<Events>();
        events.push(Event::new(entity, EventKind::Add));

        entity
    }
}

#[test]
pub fn test_random_scenery() {
    let mut game = Game::new();
    Loader::load_random_map(
        &mut game,
        &RandomMapCfg {
            size: 5,
            seed: 5,
            ships: 5,
        },
    );
}
