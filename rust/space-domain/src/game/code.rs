use bevy_ecs::prelude::*;

pub type Code = String;
pub type CodeRef = str;

/// Is a user friendly ID used to reference objects from files and user input.
///
/// It is "unique" but no constraint is enforced
// TODO: enforce uniqueness
#[derive(Component)]
pub struct HasCode {
    pub code: Code,
}

impl HasCode {
    pub fn new(code: Code) -> Self {
        Self { code }
    }

    pub fn from_str(str: &str) -> Self {
        Self {
            code: str.to_string(),
        }
    }
}

// pub fn get_entity_by_code(world: &mut World, code: &CodeRef) -> Option<Entity> {
//     world.run_system_once_with(code, find)
// }

pub fn find_entity_by_code(
    In(code): In<String>,
    query: Query<(Entity, &HasCode)>,
) -> Option<Entity> {
    query
        .iter()
        .find(|(_, c)| c.code.eq_ignore_ascii_case(code.as_str()))
        .map(|(e, _)| e)
}
