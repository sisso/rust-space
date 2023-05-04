use specs::prelude::*;

pub type Code = String;
pub type CodeRef = str;

#[derive(Component)]
pub struct HasCode {
    pub code: Code,
}

pub fn get_entity_by_code(world: &World, code: &CodeRef) -> Option<Entity> {
    (&world.entities(), &world.read_storage::<HasCode>())
        .join()
        .find(|(_, c)| c.code.eq_ignore_ascii_case(code))
        .map(|(e, _)| e)
}
