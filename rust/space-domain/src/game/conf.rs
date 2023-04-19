use serde::{Deserialize, Serialize};
use space_galaxy::system_generator::UniverseCfg;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conf {
    pub system_generator: UniverseCfg,
    pub prefabs: Prefabs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prefabs {
    pub wares: Vec<Ware>,
    pub receipts: Vec<Receipt>,
    pub fleets: Vec<Fleet>,
    pub stations: Vec<Station>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ware {
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptWare {
    pub ware: String,
    pub amount: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub code: String,
    pub label: String,
    pub input: Vec<ReceiptWare>,
    pub output: Vec<ReceiptWare>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fleet {
    pub code: String,
    pub label: String,
    pub speed: f32,
    pub storage: f32,
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
    consumes_ware: String,
    consumes_amount: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Factory {
    receipt: String,
}

pub fn load_str(buffer: &str) -> Result<Conf, String> {
    let result = commons::hocon::load_str(buffer);
    result.map_err(|err| format!("fail to load config from str by: {:?}", err))
}
