use crate::game::prefab::PrefabId;
use crate::game::wares::WareAmount;
use crate::utils::DeltaTime;

/// Blue print is a receipt owned by a builder (like shipyard) that define how it can build things
/// on required resources, production time and what prefab will be produced.
#[derive(Debug, Clone)]
pub struct Blueprint {
    pub label: String,
    pub input: Vec<WareAmount>,
    pub output: PrefabId,
    pub time: DeltaTime,
}
