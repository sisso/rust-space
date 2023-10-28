use specs::prelude::*;

/// Other objects can dock in this object
#[derive(Debug, Clone, Component, Default)]
pub struct Docking {
    pub docked: Vec<Entity>,
}
