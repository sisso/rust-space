use cgmath::{prelude::*, vec2};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler};
use ggez::{graphics, timer, Context, ContextBuilder, GameResult};
use specs::prelude::*;
use specs::{World, WorldExt};
use std::borrow::BorrowMut;

#[derive(Clone, Debug)]
struct Moveable {
    pos: cgmath::Point2<f32>,
    max_speed: f32,
    vel: cgmath::Vector2<f32>,
}

impl Component for Moveable {
    type Storage = VecStorage<Self>;
}

#[derive(Clone, Debug)]
struct MoveableHistoric {
    list: Vec<Moveable>,
}

impl Component for MoveableHistoric {
    type Storage = VecStorage<Self>;
}

#[derive(Clone, Debug)]
struct MoveCommand {
    to: cgmath::Point2<f32>,
}

impl Component for MoveCommand {
    type Storage = VecStorage<Self>;
}

#[derive(Clone, Debug)]
struct PatrolCommand {
    index: usize,
    route: Vec<cgmath::Point2<f32>>,
}

impl PatrolCommand {
    pub fn next(&mut self) -> cgmath::Point2<f32> {
        let value = self.route[self.index];
        self.index += 1;
        if self.index >= self.route.len() {
            self.index = 0;
        }
        value
    }
}

impl Component for PatrolCommand {
    type Storage = VecStorage<Self>;
}

#[derive(Clone, Debug)]
struct Ship {
    size: f32,
}

impl Component for Ship {
    type Storage = VecStorage<Self>;
}

struct App {
    world: World,
}

impl App {
    pub fn new(ctx: &mut Context) -> GameResult<App> {
        // create world
        let mut world = World::new();
        world.register::<Ship>();
        world.register::<Moveable>();
        world.register::<MoveableHistoric>();
        world.register::<MoveCommand>();
        world.register::<PatrolCommand>();

        // add elements
        {
            world
                .create_entity()
                .with(Ship { size: 4.0 })
                .with(Moveable {
                    pos: cgmath::Point2::new(400.0, 300.0),
                    max_speed: 100.0,
                    vel: cgmath::Vector2::new(0.0, 0.0),
                })
                .with(PatrolCommand {
                    index: 0,
                    route: vec![
                        cgmath::Point2::new(200.0, 300.0),
                        cgmath::Point2::new(600.0, 300.0),
                    ],
                })
                .build();
        }

        // world.setup();

        let game = App { world };

        Ok(game)
    }
}

fn commands_system(world: &mut World) -> GameResult<()> {
    let entities = world.entities();
    let mut patrols = world.write_storage::<PatrolCommand>();
    let mut move_commands = world.write_storage::<MoveCommand>();
    let mut movables = world.write_storage::<Moveable>();

    for (entity, patrol) in (&*entities, &mut patrols).join() {
        if move_commands.get(entity).is_some() {
            continue;
        }

        let pos = patrol.next();
        println!("{:?} next pos {:?}", entity, pos);
        move_commands.insert(entity, MoveCommand { to: pos });
    }

    for (entity, movable) in (&*entities, &mut movables).join() {
        let move_command = if let Some(value) = move_commands.get_mut(entity) {
            value
        } else {
            continue;
        };

        let movable: &mut Moveable = movable;

        let delta = move_command.to - movable.pos;
        let distance = delta.magnitude();
        if distance < 0.1 {
            // println!("{:?} complete", entity);
            movable.vel = vec2(0.0, 0.0);
            move_commands.remove(entity);
        } else {
            let dir = delta.normalize();
            let speed = movable.max_speed.min(distance * 10.0);
            movable.vel = dir * speed;
            // println!("{:?} set vel {:?}", entity, movable);
        }
    }

    Ok(())
}

fn movable_system(delta: f32, world: &mut World) -> GameResult<()> {
    let mut movables = world.write_storage::<Moveable>();

    for (movable,) in (&mut movables,).join() {
        // println!("{:?} moving at {}", movable, delta);
        movable.pos = movable.pos + movable.vel * delta;
    }

    Ok(())
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let delta = timer::delta(ctx).as_secs_f32();
        commands_system(&mut self.world);
        movable_system(delta, &mut self.world);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        let ships = &self.world.read_storage::<Ship>();
        let movables = &self.world.read_storage::<Moveable>();

        for (s, m) in (ships, movables).join() {
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                m.pos,
                s.size,
                0.1,
                graphics::WHITE,
            )?;
            graphics::draw(ctx, &circle, graphics::DrawParam::default())?;
        }

        graphics::present(ctx)
    }
}

fn main() -> GameResult<()> {
    // Make a Context.
    let mut window_mode: WindowMode = Default::default();
    window_mode.resizable = true;

    let (mut ctx, mut event_loop) = ContextBuilder::new("my_game", "Cool Game Author")
        .window_mode(window_mode)
        .build()
        .expect("aieee, could not create ggez context!");

    let mut app = App::new(&mut ctx)?;

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut app) {
        Ok(_) => {
            println!("Exited cleanly.");
            Ok(())
        }
        Err(e) => {
            println!("Error occured: {}", e);
            Err(e)
        }
    }
}
