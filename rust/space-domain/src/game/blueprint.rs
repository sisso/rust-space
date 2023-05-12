use crate::game::prefab::PrefabId;
use crate::game::wares::WareAmount;
use crate::utils::DeltaTime;

#[derive(Debug, Clone)]
pub struct Blueprint {
    pub label: String,
    pub input: Vec<WareAmount>,
    pub output: PrefabId,
    pub time: DeltaTime,
}
