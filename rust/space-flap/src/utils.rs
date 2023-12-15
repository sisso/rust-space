use crate::Id;
use bevy_ecs::prelude::*;
use space_domain::game::Game;

// real encoding of a entity
pub(crate) fn proper_encode_entity(entity: Entity) -> u64 {
    let high: u32 = entity.index();
    let low: u32 = entity.generation();

    let encoded: u64 = (high as u64) << 32 | (low as u64);
    return encoded;
}

// real decoding of a entity
pub(crate) fn proper_decode_entity(value: u64) -> (u32, u32) {
    let high = (value >> 32) as u32;
    let low = (value & 0xffffffff) as u32;
    (high, low)
}

// pretty but broken encode of entity
pub(crate) fn encode_entity(entity: Entity) -> u64 {
    let high = entity.generation() as u64 * 1_000_000;
    let low = entity.index() as u64;
    return high + low;
}

// pretty but broken decode of entity
pub(crate) fn decode_entity(value: u64) -> (u32, u32) {
    let high = value / 1_000_000;
    let low = value % 1_000_000;
    (low as u32, high as u32)
}

pub(crate) fn decode_entity_and_get(g: &Game, id: Id) -> Option<Entity> {
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
            return None;
        }
        None => {
            log::warn!("get_obj for {}/{} fail, entity not found", index, egen,);
            return None;
        }
    }
}
