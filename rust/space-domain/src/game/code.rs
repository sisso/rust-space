use specs::prelude::*;

pub type Code = String;
pub type CodeRef = str;

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

pub fn get_entity_by_code(world: &World, code: &CodeRef) -> Option<Entity> {
    find(&world.entities(), &world.read_storage(), code)
}

pub fn find(entities: &Entities, codes: &ReadStorage<'_, HasCode>, code: &str) -> Option<Entity> {
    (entities, codes)
        .join()
        .find(|(_, c)| c.code.eq_ignore_ascii_case(code))
        .map(|(e, _)| e)
}
