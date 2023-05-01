use crate::game::wares::WareId;
use serde::{Deserialize, Serialize};
use space_galaxy::system_generator::UniverseCfg;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conf {
    pub system_generator: UniverseCfg,
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
    pub fleets: Vec<Fleet>,
    pub stations: Vec<Station>,
}

impl Prefabs {
    pub fn find_were_id_by_code(&self, code: &str) -> Option<WareId> {
        self.wares
            .iter()
            .enumerate()
            .find(|(_, ware)| ware.code.as_str() == code)
            .map(|(id, _)| id)
    }

    pub fn get_by_ware_id(&self, ware_id: WareId) -> &Ware {
        self.wares.get(ware_id).expect("fail to find ware by id")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Ware {
    pub code: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptWare {
    pub ware: String,
    pub amount: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub code: String,
    pub label: String,
    pub input: Vec<ReceiptWare>,
    pub output: Vec<ReceiptWare>,
    pub time: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fleet {
    pub code: String,
    pub label: String,
    pub speed: f32,
    pub storage: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub code: String,
    pub label: String,
    pub storage: f32,
    pub shipyard: Option<Shipyard>,
    pub factory: Option<Factory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shipyard {
    pub consumes_ware: String,
    pub consumes_amount: u32,
    pub time: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Factory {
    pub receipt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Params {
    pub prefab_station_shipyard: String,
    pub prefab_station_factory: String,
    pub prefab_station_solar: String,
    pub prefab_ship_trade: String,
    pub prefab_ship_miner: String,
}
