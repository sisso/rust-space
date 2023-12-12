use crate::game::bevy_utils::WorldExt;
use crate::game::commands::Command;
use crate::game::factory::Receipt;
use crate::game::loader::Loader;
use crate::game::objects::ObjId;
use crate::game::sectors::SectorId;
use crate::game::shipyard::{ProductionOrder, Shipyard};
use crate::game::utils::{DeltaTime, V2};
use crate::game::wares::{WareAmount, WareId};
use crate::game::{code, sectors, wares, Game};
use bevy_ecs::prelude::*;
use bevy_ecs::system::{RunSystemOnce, SystemState};
use commons::math::P2I;

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

pub struct SceneryBuilder;

#[derive(Default)]
pub struct SceneryBuilderInit {
    tasks: Vec<Box<dyn BuilderTask>>,
}

#[derive(Default, Debug)]
pub struct SceneryBuilderResult {
    pub sectors: Vec<SectorId>,
    pub wares: Vec<WareId>,
    pub fleets: Vec<ObjId>,
    pub stations: Vec<ObjId>,
    pub asteroids: Vec<ObjId>,
    pub prefabs: Vec<ObjId>,
}

pub trait BuilderTask {
    fn apply(&self, game: &mut Game, result: &mut SceneryBuilderResult);
}

pub trait BuilderStep {
    fn get_tasks(&self) -> &Vec<Box<dyn BuilderTask>>;
    fn add_task(&mut self, task: Box<dyn BuilderTask>);
}

pub trait BuilderBuild: BuilderStep {
    fn build(&self, game: &mut Game) -> SceneryBuilderResult {
        let mut rs = SceneryBuilderResult::default();
        for task in self.get_tasks() {
            task.apply(game, &mut rs);
        }
        rs
    }
}

impl SceneryBuilder {
    pub fn new() -> SceneryBuilderInit {
        SceneryBuilderInit::default()
    }
}

impl SceneryBuilderInit {
    pub fn add_ware<T: Into<String>>(mut self, code: T) -> Self {
        struct Task {
            code: String,
        }
        impl BuilderTask for Task {
            fn apply(&self, game: &mut Game, result: &mut SceneryBuilderResult) {
                game.world.run_commands(|mut commands| {
                    let ware_ore_id =
                        Loader::add_ware(&mut commands, self.code.clone(), self.code.clone());
                    result.wares.extend(vec![ware_ore_id]);
                })
            }
        }
        self.tasks.push(Box::new(Task { code: code.into() }));
        self
    }

    pub fn basic_wares(mut self) -> Self {
        struct Task {}
        impl BuilderTask for Task {
            fn apply(&self, game: &mut Game, result: &mut SceneryBuilderResult) {
                game.world.run_commands(|mut commands| {
                    let ware_ore_id = Loader::add_ware(&mut commands, "ore", "Ore");
                    let ware_components_id =
                        Loader::add_ware(&mut commands, "components", "Components");
                    result.wares.extend(vec![ware_ore_id, ware_components_id]);
                })
            }
        }
        self.tasks.push(Box::new(Task {}));
        self
    }

    pub fn builder_single_sector(mut self) -> SceneryBuilderWithSector {
        struct Task {}
        impl BuilderTask for Task {
            fn apply(&self, game: &mut Game, result: &mut SceneryBuilderResult) {
                game.world.run_commands(|mut commands| {
                    let sector_id =
                        Loader::add_sector(&mut commands, P2I::new(0, 0), "Sector".to_string());
                    result.sectors = vec![sector_id];
                });

                game.world
                    .run_system_once(sectors::system_update_sectors_index);
            }
        }
        self.tasks.push(Box::new(Task {}));
        SceneryBuilderWithSector { tasks: self.tasks }
    }

    pub fn builder_two_sectors(mut self) -> SceneryBuilderWithSector {
        struct Task {}
        impl BuilderTask for Task {
            fn apply(&self, game: &mut Game, result: &mut SceneryBuilderResult) {
                let (sector_0, sector_1) = game.world.run_commands(|mut commands| {
                    let sector_0 =
                        Loader::add_sector(&mut commands, P2I::new(0, 0), "Sector 0".to_string());
                    let sector_1 =
                        Loader::add_sector(&mut commands, P2I::new(1, 0), "Sector 1".to_string());

                    Loader::add_jump(
                        &mut commands,
                        sector_0,
                        V2::new(0.5, 0.3),
                        sector_1,
                        V2::new(0.0, 0.0),
                    );

                    (sector_0, sector_1)
                });

                game.world
                    .run_system_once(sectors::system_update_sectors_index);
                result.sectors = vec![sector_0, sector_1];
            }
        }
        self.tasks.push(Box::new(Task {}));
        SceneryBuilderWithSector { tasks: self.tasks }
    }

    pub fn add_fleets_prefabs(mut self, ware: &str) -> SceneryBuilderInit {
        struct Task {
            ware_code: String,
        }
        impl BuilderTask for Task {
            fn apply(&self, game: &mut Game, result: &mut SceneryBuilderResult) {
                let ware_id = game
                    .world
                    .run_system_once_with(self.ware_code.clone(), code::find_entity_by_code)
                    .expect("ware not found");

                game.world.run_commands(|mut commands| {
                    let (trader_id, miner_id) =
                        load_sceneries_fleets_prefabs(&mut commands, ware_id);
                    result.prefabs.extend(vec![trader_id, miner_id]);
                });
            }
        }
        self.tasks.push(Box::new(Task {
            ware_code: ware.to_string(),
        }));
        self
    }
}

fn load_sceneries_fleets_prefabs(commands: &mut Commands, ware_id: ObjId) -> (Entity, Entity) {
    let new_obj = Loader::new_ship(2.0, "Trade fleet".to_string())
        .with_command(Command::trade())
        .with_production_cost(5.0, vec![WareAmount::new(ware_id, 50)]);
    let trade_id = Loader::add_prefab(commands, "trade_fleet", "Trade Fleet", new_obj, true, false);

    let new_obj = Loader::new_ship(2.0, "Mine fleet".to_string())
        .with_command(Command::mine())
        .with_production_cost(5.0, vec![WareAmount::new(ware_id, 50)]);

    let miner_id = Loader::add_prefab(commands, "mine_fleet", "Mine fleet", new_obj, true, false);

    (trade_id, miner_id)
}

#[derive(Default)]
pub struct SceneryBuilderWithSector {
    tasks: Vec<Box<dyn BuilderTask>>,
}

impl BuilderStep for SceneryBuilderWithSector {
    fn get_tasks(&self) -> &Vec<Box<dyn BuilderTask>> {
        &self.tasks
    }

    fn add_task(&mut self, task: Box<dyn BuilderTask>) {
        self.tasks.push(task);
    }
}

impl BuilderBuild for SceneryBuilderWithSector {}

impl SceneryBuilderWithSector {
    pub fn add_asteroid(mut self, sector_i: usize, ware_id: usize, pos: V2) -> Self {
        struct Task {
            sector_i: usize,
            ware_i: usize,
            pos: V2,
        }
        impl BuilderTask for Task {
            fn apply(&self, game: &mut Game, result: &mut SceneryBuilderResult) {
                game.world.run_commands(|mut commands| {
                    let ware_ore_id = result.wares.get(self.ware_i).expect("ware not found");
                    let sector_id = result.sectors.get(self.sector_i).expect("sector not found");
                    let asteroid_id =
                        Loader::add_asteroid(&mut commands, *sector_id, self.pos, *ware_ore_id);
                    result.asteroids.push(asteroid_id);
                });
            }
        }
        self.tasks.push(Box::new(Task {
            sector_i,
            ware_i: ware_id,
            pos,
        }));
        self
    }

    pub fn new_mothership(self) -> SceneryBuilderMothership<SceneryBuilderWithSector> {
        SceneryBuilderMothership::new(self)
    }

    pub fn add_miner(mut self) -> SceneryBuilderWithSector {
        struct Task {}
        impl BuilderTask for Task {
            fn apply(&self, game: &mut Game, result: &mut SceneryBuilderResult) {
                game.world.run_commands(|mut commands| {
                    let station_id = result.stations.get(0).expect("no station found");
                    let fleet_id =
                        Loader::add_ship_miner(&mut commands, *station_id, 2.0, "miner".into());
                    result.fleets.push(fleet_id);
                });
            }
        }
        self.tasks.push(Box::new(Task {}));
        self
    }
}

pub struct SceneryBuilderMothership<T: BuilderStep> {
    previous: T,
    random_orders: bool,
}

impl<T: BuilderStep> SceneryBuilderMothership<T> {
    pub fn new(previous: T) -> Self {
        Self {
            previous,
            random_orders: false,
        }
    }

    pub fn with_random_orders(mut self) -> Self {
        self.random_orders = true;
        self
    }

    pub fn build(mut self) -> T {
        struct Task {
            random_orders: bool,
        }
        impl BuilderTask for Task {
            fn apply(&self, game: &mut Game, result: &mut SceneryBuilderResult) {
                let ware_input_id = result.wares.get(0).expect("not ware input found");
                let ware_output_id = result.wares.get(1).expect("not ware output found");

                let sector_id = result.sectors.get(0).expect("no sector found");
                let receipt = Receipt {
                    label: "mothership production".to_string(),
                    input: vec![WareAmount::new(*ware_input_id, 1)],
                    output: vec![WareAmount::new(*ware_output_id, 1)],
                    time: DeltaTime::from(1.0),
                };

                let mothership_id = game.world.run_commands(|mut commands| {
                    Loader::add_mothership(&mut commands, *sector_id, V2::new(0.0, 0.0), receipt)
                });

                if self.random_orders {
                    game.world
                        .get_entity_mut(mothership_id)
                        .expect("mothership_id not found")
                        .get_mut::<Shipyard>()
                        .expect("mothership has no shipyard")
                        .set_production_order(ProductionOrder::Random);
                }

                result.stations.push(mothership_id);
            }
        }
        self.previous.add_task(Box::new(Task {
            random_orders: self.random_orders,
        }));
        self.previous
    }
}

pub struct MinimumScenery {
    pub ware_ore_id: Entity,
    pub asteroid_id: Entity,
    pub sector_0: Entity,
}

/// Minimum scenery, a sector
///
/// Is defined as a simple:
/// - ore ware
/// - 1 sector,
/// - asteroid (ore)
pub fn load_minimum_scenery(game: &mut Game) -> MinimumScenery {
    let rs = SceneryBuilder::new()
        .add_ware("ore")
        .builder_single_sector()
        .add_asteroid(0, 0, V2::new(2.0, 0.0))
        .build(game);

    MinimumScenery {
        asteroid_id: rs.asteroids[0],
        sector_0: rs.sectors[0],
        ware_ore_id: rs.wares[0],
    }
}

/// Basic scenery used for testing and samples
///
/// Is defined as a simple:
/// - 2 sector,
/// - miner ship
/// - trade ship
/// - factory (ore -> components)
/// - shipyard
/// - asteroid (ore)
pub fn load_basic_scenery(game: &mut Game) -> BasicScenery {
    let scenery = game.world.run_commands(|mut commands| {
        // init wares
        let ware_ore_id = Loader::add_ware(&mut commands, "ore".to_string(), "Ore".to_string());
        let ware_components_id = Loader::add_ware(
            &mut commands,
            "components".to_string(),
            "Components".to_string(),
        );

        // receipts
        let ore_processing_receipt = Receipt {
            label: "ore processing".to_string(),
            input: vec![WareAmount::new(ware_ore_id, 20)],
            output: vec![WareAmount::new(ware_components_id, 10)],
            time: DeltaTime(1.0),
        };

        // init prefabs
        load_sceneries_fleets_prefabs(&mut commands, ware_components_id);

        // init sectors
        let sector_0 = Loader::add_sector(&mut commands, P2I::new(0, 0), "Sector 0".to_string());
        let sector_1 = Loader::add_sector(&mut commands, P2I::new(1, 0), "Sector 1".to_string());

        Loader::add_jump(
            &mut commands,
            sector_0,
            V2::new(0.5, 0.3),
            sector_1,
            V2::new(0.0, 0.0),
        );

        // init objects
        let asteroid_id =
            Loader::add_asteroid(&mut commands, sector_1, V2::new(-2.0, 3.0), ware_ore_id);
        let component_factory_id = Loader::add_factory(
            &mut commands,
            sector_0,
            V2::new(3.0, -1.0),
            ore_processing_receipt,
        );

        let shipyard_id = Loader::add_shipyard(&mut commands, sector_0, V2::new(1.0, -3.0));
        let miner_id = Loader::add_ship_miner(&mut commands, shipyard_id, 2.0, "miner".to_string());
        let trader_id = Loader::add_ship_trader(
            &mut commands,
            component_factory_id,
            2.0,
            "trader".to_string(),
        );

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
    });

    // update caches
    game.world
        .run_system_once(sectors::system_update_sectors_index);

    // set shipyard to build random stuff
    Loader::set_shipyard_order_to_random(&mut game.world, scenery.shipyard_id)
        .expect("fail to set shipyard order");

    scenery
}

/// Advanced scenery used for testing and samples
///
/// Is defined as a simple:
/// - 2 sector,
/// - miner ship
/// - trade ship
/// - solar station (energy)
/// - factory (ore + energy -> components)
/// - shipyard
/// - 3x asteroid (ore)
pub fn load_advanced_scenery(world: &mut World) {
    world.run_commands(|mut commands| {
        // init wares
        let ware_ore_id = Loader::add_ware(&mut commands, "ore".to_string(), "Ore".to_string());
        let ware_components_id = Loader::add_ware(
            &mut commands,
            "components".to_string(),
            "Components".to_string(),
        );
        let ware_energy =
            Loader::add_ware(&mut commands, "energy".to_string(), "Energy".to_string());

        // init prefabs
        load_sceneries_fleets_prefabs(&mut commands, ware_components_id);

        // receipts
        let receipt_process_ores = Receipt {
            label: "ore processing".to_string(),
            input: vec![
                WareAmount::new(ware_ore_id, 20),
                WareAmount::new(ware_energy, 10),
            ],
            output: vec![WareAmount::new(ware_components_id, 10)],
            time: DeltaTime(1.0),
        };
        let receipt_produce_energy = Receipt {
            label: "solar power".to_string(),
            input: vec![],
            output: vec![WareAmount::new(ware_energy, 10)],
            time: DeltaTime(5.0),
        };

        // init sectors
        let sector_0 = Loader::add_sector(&mut commands, P2I::new(0, 0), "sector 0".to_string());
        let sector_1 = Loader::add_sector(&mut commands, P2I::new(1, 0), "sector 1".to_string());

        Loader::add_jump(
            &mut commands,
            sector_0,
            V2::new(0.5, 0.3),
            sector_1,
            V2::new(0.0, 0.0),
        );

        // init objects
        Loader::add_asteroid(&mut commands, sector_1, V2::new(-2.0, 3.0), ware_ore_id);
        Loader::add_asteroid(&mut commands, sector_1, V2::new(-2.2, 2.8), ware_ore_id);
        Loader::add_asteroid(&mut commands, sector_1, V2::new(-2.8, 3.1), ware_ore_id);

        let component_factory_id = Loader::add_factory(
            &mut commands,
            sector_0,
            V2::new(3.0, -1.0),
            receipt_process_ores,
        );

        let _energy_factory_id = Loader::add_factory(
            &mut commands,
            sector_0,
            V2::new(-0.5, 1.5),
            receipt_produce_energy,
        );

        let shipyard_id = Loader::add_shipyard(&mut commands, sector_0, V2::new(1.0, -3.0));
        Loader::add_ship_miner(&mut commands, shipyard_id, 2.0, "miner".to_string());
        Loader::add_ship_trader(
            &mut commands,
            component_factory_id,
            2.0,
            "trader".to_string(),
        );
    });

    world.run_system_once(sectors::system_update_sectors_index);
}

pub struct MothershipScenery {
    pub sector_id: SectorId,
    pub miner_id: ObjId,
    pub mothership_id: ObjId,
    pub asteroid_id: ObjId,
}

/// Basic scenery with mothership
///
/// Is defined as a simple:
/// - 2 sector,
/// - miner ship
/// - mothership (ore -> components) and shipyard
/// - asteroid (ore)
pub fn load_basic_mothership_scenery(game: &mut Game) -> MothershipScenery {
    let rs = new_basic_mothership_scenery().build(game);

    MothershipScenery {
        sector_id: rs.sectors[0],
        miner_id: rs.fleets[0],
        mothership_id: rs.stations[0],
        asteroid_id: rs.asteroids[0],
    }
}

pub fn new_basic_mothership_scenery() -> SceneryBuilderWithSector {
    SceneryBuilder::new()
        .add_ware("ore")
        .add_ware("components")
        .add_fleets_prefabs("components")
        .builder_single_sector()
        .add_asteroid(0, 0, V2::new(2.0, 0.0))
        .new_mothership()
        .with_random_orders()
        .build()
        .add_miner()
}
