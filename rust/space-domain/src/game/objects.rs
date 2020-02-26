use crate::utils::*;
use specs::prelude::*;
use std::collections::HashMap;

pub type ObjId = Entity;

// TODO: merge with station
#[derive(Debug, Copy, Clone, Component)]
pub struct HasDock;

pub struct Objects;
