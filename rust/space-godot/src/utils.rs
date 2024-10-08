use crate::game_api::Id;
use godot::prelude::*;
use space_domain::game::game::Game;
use space_domain::game::objects::ObjId;

// real encoding of a entity
pub fn proper_encode_entity(entity: ObjId) -> u64 {
    let high: u32 = entity.index();
    let low: u32 = entity.generation();

    let encoded: u64 = (high as u64) << 32 | (low as u64);
    return encoded;
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
    return high + low;
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
            return None;
        }
        None => {
            log::warn!("get_obj for {}/{} fail, entity not found", index, egen,);
            return None;
        }
    }
}

pub fn decode_entity_and_get(g: &Game, id: Id) -> ObjId {
    try_decode_entity_and_get(g, id).expect("invalid i")
}

// /// convert into godot variant but flatten the first dictionary to remove classname unique field
// pub fn to_godot_flat<T: ToGodot>(value: T) -> Variant {
//     let variant = value.to_variant();
//     match variant.get_type() {
//         VariantType::Dictionary => {
//             let mut dict = variant.to::<Dictionary>();
//             assert_eq!(dict.len(), 1, "variant must have at least one field");
//             let key = dict.keys_array().get(0);
//             let value = dict.get(key).expect("key must exist");
//             value
//         }
//         _ => panic!("variant must be a dictionary"),
//     }
// }

pub fn to_godot_flat_option<T: ToGodot>(value: Option<T>) -> Variant {
    if let Some(value) = value {
        value.to_variant()
    } else {
        Variant::nil()
    }
}
