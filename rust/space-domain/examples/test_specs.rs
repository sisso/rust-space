#[macro_use]
extern crate space_macros;
#[macro_use]
extern crate specs_derive;
#[macro_use]
extern crate shred_derive;

use specs::prelude::*;
use specs_derive::*;
use std::borrow::BorrowMut;

// A component contains data
// which is associated with an entity.
#[derive(Debug)]
struct Vel(f32);

impl Component for Vel {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
struct Pos(f32);

impl Component for Pos {
    type Storage = VecStorage<Self>;
}

struct SysA;

#[derive(SystemData)]
struct SysAData<'a> {
    pos: WriteStorage<'a, Pos>,
    vel: ReadStorage<'a, Vel>,
}

impl<'a> System<'a> for SysA {
    type SystemData = SysAData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        println!("running system");
        for (pos, vel) in (data.pos.borrow_mut(), &data.vel).join() {
            pos.0 += vel.0;
        }
    }
}

fn main() {
    let mut world = World::new();
//    world.register::<Pos>();
//    world.register::<Vel>();
//
//    // An entity may or may not contain some component.
//
//    world.create_entity().with(Vel(2.0)).with(Pos(0.0)).build();
//    world.create_entity().with(Vel(4.0)).with(Pos(1.6)).build();
//    world.create_entity().with(Vel(1.5)).with(Pos(5.4)).build();
//
//    // This entity does not have `Vel`, so it won't be dispatched.
//    world.create_entity().with(Pos(2.0)).build();

    // This builds a dispatcher.
    // The third parameter of `with` specifies
    // logical dependencies on other systems.
    // Since we only have one, we don't depend on anything.
    // See the `full` example for dependencies.
    let mut dispatcher = DispatcherBuilder::new();
    dispatcher.add(SysA, "sys_a", &[]);
    let mut dispatcher = dispatcher.build();
    // This will call the `setup` function of every system.
    // In this example this has no effect since we already registered our components.
    dispatcher.setup(&mut world);

    // This dispatches all the systems in parallel (but blocking).
    dispatcher.dispatch(&mut world);

    println!("done");
}