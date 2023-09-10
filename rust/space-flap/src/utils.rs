use crate::Id;
use space_domain::game::Game;
use specs::{Entity, WorldExt};

// real encoding of a entity
pub(crate) fn proper_encode_entity(entity: Entity) -> u64 {
    let high: u32 = entity.id();
    let low: i32 = entity.gen().id();

    let encoded: u64 = (high as u64) << 32 | (low as u64);
    return encoded;
}

// real decoding of a entity
pub(crate) fn proper_decode_entity(value: u64) -> (u32, i32) {
    let high = (value >> 32) as u32;
    let low = (value & 0xffffffff) as i32;
    (high, low)
}

// pretty but broken encode of entity
pub(crate) fn encode_entity(entity: Entity) -> u64 {
    let high = entity.gen().id() as u64 * 1_000_000;
    let low = entity.id() as u64;
    return high + low;
}

// pretty but broken decode of entity
pub(crate) fn decode_entity(value: u64) -> (u32, i32) {
    let high = value / 1_000_000;
    let low = value % 1_000_000;
    (low as u32, high as i32)
}

pub(crate) fn decode_entity_and_get(g: &Game, id: Id) -> Option<Entity> {
    let (eid, egen) = decode_entity(id);
    let entities = g.world.entities();
    let e = entities.entity(eid);
    if egen == e.gen().id() {
        Some(e)
    } else {
        log::warn!(
            "get_obj for {}/{} fail, entity has gen {}",
            eid,
            egen,
            e.gen().id()
        );
        return None;
    }
}
