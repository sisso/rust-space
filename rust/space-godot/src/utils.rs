use crate::game_api::Id;
use godot::prelude::*;
use space_domain::game::game::Game;
use space_domain::game::objects::ObjId;

// real encoding of a entity
pub fn proper_encode_entity(entity: ObjId) -> u64 {
    let high: u32 = entity.index();
    let low: u32 = entity.generation();

    let encoded: u64 = (high as u64) << 32 | (low as u64);
    encoded
}

// real decoding of a entity
pub fn proper_decode_entity(value: u64) -> (u32, u32) {
    let high = (value >> 32) as u32;
    let low = (value & 0xffffffff) as u32;
    (high, low)
}

// pretty but broken encode of entity
pub fn encode_entity(entity: ObjId) -> i64 {
    let high = entity.generation() as i64 * 1_000_000;
    let low = entity.index() as i64;
    high + low
}

// pretty but broken decode of entity
pub fn decode_entity(value: i64) -> (u32, u32) {
    let high = value / 1_000_000;
    let low = value % 1_000_000;
    (low as u32, high as u32)
}

pub fn try_decode_entity_and_get(g: &Game, id: Id) -> Option<ObjId> {
    let (index, egen) = decode_entity(id);
    let entities = g.world.entities();
    match entities.resolve_from_id(index) {
        Some(e) if e.generation() == egen => Some(e),
        Some(e) => {
            log::warn!(
                "get_obj for {}/{} fail, entity has generation {}",
                index,
                egen,
                e.generation()
            );
            None
        }
        None => {
            log::warn!("get_obj for {}/{} fail, entity not found", index, egen,);
            None
        }
    }
}

pub fn decode_entity_and_get(g: &Game, id: Id) -> ObjId {
    try_decode_entity_and_get(g, id).expect("invalid i")
}

pub fn to_godot_flat_option<T: ToGodot>(value: Option<T>) -> Variant {
    if let Some(value) = value {
        value.to_variant()
    } else {
        Variant::nil()
    }
}
