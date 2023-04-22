use specs::prelude::*;

#[derive(Component)]
pub struct Code {
    pub code: String,
}

pub fn get_entity_by_code(world: &World, code: &str) -> Option<Entity> {
    (&world.entities(), &world.read_storage::<Code>())
        .join()
        .find(|(_, c)| c.code.eq_ignore_ascii_case(code))
        .map(|(e, _)| e)
}
