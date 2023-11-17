use specs::prelude::*;

/// Other objects can dock in this object
#[derive(Debug, Clone, Component, Default)]
pub struct HasDocking {
    pub docked: Vec<Entity>,
}
