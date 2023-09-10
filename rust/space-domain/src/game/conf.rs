use serde::{Deserialize, Serialize};

use space_galaxy::system_generator::UniverseCfg;

pub type BlueprintCode = String;
pub type Code = String;
pub type Label = String;
pub type WareCode = String;
pub type FleetCode = String;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Conf {
    pub system_generator: Option<UniverseCfg>,
    pub prefabs: Prefabs,
    pub params: Params,
}

pub fn load_str(buffer: &str) -> Result<Conf, String> {
    let result = commons::hocon::load_str(buffer);
    result.map_err(|err| format!("fail to load config from str by: {:?}", err))
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Prefabs {
    pub wares: Vec<Ware>,
    pub receipts: Vec<Receipt>,
    pub blueprints: Vec<Blueprint>,
    pub fleets: Vec<Fleet>,
    pub stations: Vec<Station>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Ware {
    pub code: WareCode,
    pub label: Label,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptWare {
    pub ware: Code,
    pub amount: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub code: Code,
    pub label: Label,
    pub input: Vec<ReceiptWare>,
    pub output: Vec<ReceiptWare>,
    pub time: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    pub code: BlueprintCode,
    pub label: Label,
    pub input: Vec<ReceiptWare>,
    pub output: FleetCode,
    pub time: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fleet {
    pub code: FleetCode,
    pub label: Label,
    pub speed: f32,
    pub storage: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub code: Code,
    pub label: Label,
    pub storage: f32,
    pub shipyard: Option<Shipyard>,
    pub factory: Option<Factory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shipyard {
    pub blueprints: Vec<BlueprintCode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Factory {
    pub receipt: Code,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Params {
    pub prefab_station_shipyard: Code,
    pub prefab_station_factory: Code,
    pub prefab_station_solar: Code,
    pub prefab_ship_trade: FleetCode,
    pub prefab_ship_miner: FleetCode,
    pub player_blueprints: Vec<Code>,
}
