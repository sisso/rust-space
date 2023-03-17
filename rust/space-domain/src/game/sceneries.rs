use crate::game::factory::Receipt;
use crate::game::loader::{BasicScenery, Loader};
use crate::game::wares::WareAmount;
use crate::game::{sectors, Game};
use crate::utils::{DeltaTime, V2};
use shred::World;

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
            input: vec![WareAmount::new(ware_ore_id, 20)],
            output: vec![WareAmount::new(ware_components_id, 10)],
            time: DeltaTime(1.0),
        },
    );
    let shipyard_id = Loader::add_shipyard(world, sector_0, V2::new(1.0, -3.0), ware_components_id);
    let miner_id = Loader::add_ship_miner(world, shipyard_id, 2.0, "miner".to_string());
    let trader_id = Loader::add_ship_trader(world, component_factory_id, 2.0, "trader".to_string());

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
        input: vec![
            WareAmount::new(ware_ore_id, 20),
            WareAmount::new(ware_energy, 10),
        ],
        output: vec![WareAmount::new(ware_components_id, 10)],
        time: DeltaTime(1.0),
    };
    let receipt_produce_energy = Receipt {
        input: vec![],
        output: vec![WareAmount::new(ware_energy, 10)],
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

    let shipyard_id = Loader::add_shipyard(world, sector_0, V2::new(1.0, -3.0), ware_components_id);
    Loader::add_ship_miner(world, shipyard_id, 2.0, "miner".to_string());
    Loader::add_ship_trader(world, component_factory_id, 2.0, "trader".to_string());
}
