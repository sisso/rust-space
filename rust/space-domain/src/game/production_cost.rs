use crate::game::wares::WareAmount;
use crate::game::work::WorkUnit;
use specs::prelude::*;

/// How much cost to build this unit/prefab
#[derive(Clone, Debug, Component)]
pub struct ProductionCost {
    pub cost: Vec<WareAmount>,
    pub work: WorkUnit,
}
