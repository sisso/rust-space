use specs::{Component as SpecComponent, EntityBuilder, SystemData, WriteStorage};

pub trait BuilderExtra {
    fn set<C: SpecComponent + Send + Sync>(&mut self, c: C);
}

impl<'a> BuilderExtra for EntityBuilder<'a> {
    #[inline]
    fn set<T: SpecComponent>(&mut self, c: T) {
        {
            let mut storage: WriteStorage<T> = SystemData::fetch(&self.world);
            // This can't fail.  This is guaranteed by the lifetime 'a
            // in the EntityBuilder.
            storage.insert(self.entity, c).unwrap();
        }
    }
}
