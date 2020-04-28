use cgmath::{prelude::*, vec2, vec3, Deg, Euler, Quaternion, Rad, Vector2};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler};
use ggez::{graphics, timer, Context, ContextBuilder, GameResult};
use specs::prelude::*;
use specs::{World, WorldExt};
use specs_derive::Component;
use std::borrow::BorrowMut;
use std::ops::Deref;

#[derive(Clone, Debug, Component)]
struct Cfg {
    speed_reduction: f32,
}

#[derive(Clone, Debug, Component)]
struct Moveable {
    pos: cgmath::Point2<f32>,
    max_speed: f32,
    vel: cgmath::Vector2<f32>,
}

#[derive(Clone, Debug, Component)]
struct MoveableHistoric {
    list: Vec<Moveable>,
}

#[derive(Clone, Debug, Component)]
struct MoveCommand {
    to: cgmath::Point2<f32>,
}

#[derive(Clone, Debug, Component)]
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

#[derive(Clone, Debug, Component)]
struct FollowCommand {
    target: Entity,
    pos: cgmath::Vector2<f32>,
}

#[derive(Clone, Debug, Component)]
struct Model {
    size: f32,
    color: graphics::Color,
}

struct App {
    world: World,
}

impl App {
    pub fn new(ctx: &mut Context) -> GameResult<App> {
        // create world
        let mut world = World::new();
        world.register::<Model>();
        world.register::<Moveable>();
        world.register::<MoveableHistoric>();
        world.register::<MoveCommand>();
        world.register::<PatrolCommand>();
        world.register::<FollowCommand>();
        world.register::<Cfg>();

        world.insert(Cfg {
            speed_reduction: 9.0,
        });

        // add elements
        {
            let entity_0 = world
                .create_entity()
                .with(Model {
                    size: 6.0,
                    color: graphics::WHITE,
                })
                .with(Moveable {
                    pos: cgmath::Point2::new(400.0, 300.0),
                    max_speed: 54.0,
                    vel: cgmath::Vector2::new(0.0, 0.0),
                })
                .with(PatrolCommand {
                    index: 0,
                    route: vec![
                        cgmath::Point2::new(200.0, 300.0),
                        cgmath::Point2::new(400.0, 150.0),
                        cgmath::Point2::new(600.0, 300.0),
                        cgmath::Point2::new(400.0, 550.0),
                    ],
                })
                .build();

            world
                .create_entity()
                .with(Model {
                    size: 2.0,
                    color: graphics::Color::new(1.0, 0.0, 0.0, 1.0),
                })
                .with(Moveable {
                    pos: cgmath::Point2::new(450.0, 320.0),
                    max_speed: 60.0,
                    vel: cgmath::Vector2::new(0.0, 0.0),
                })
                .with(FollowCommand {
                    target: entity_0,
                    pos: vec2(0.0, -10.0),
                })
                .build();

            world
                .create_entity()
                .with(Model {
                    size: 2.0,
                    color: graphics::Color::new(0.0, 1.0, 0.0, 1.0),
                })
                .with(Moveable {
                    pos: cgmath::Point2::new(450.0, 320.0),
                    max_speed: 55.0,
                    vel: cgmath::Vector2::new(0.0, 0.0),
                })
                .with(FollowCommand {
                    target: entity_0,
                    pos: vec2(0.0, 10.0),
                })
                .build();
        }

        let game = App { world };

        Ok(game)
    }
}

fn follow_system(world: &mut World) -> GameResult<()> {
    let entities = &world.entities();
    let follow_commands = &mut world.write_storage::<FollowCommand>();
    let move_to_commands = &mut world.write_storage::<MoveCommand>();
    let movables = &world.read_storage::<Moveable>();

    //  collect for each follow the target position
    for (entity, follow) in (entities, follow_commands).join() {
        let target_movable = if let Some(m) = (&movables).get(follow.target) {
            m
        } else {
            continue;
        };

        // update movable with target position
        let mut relative_pos = rotate_vector(target_movable.vel.normalize(), follow.pos);
        if relative_pos.x.is_nan() || relative_pos.y.is_nan() {
            relative_pos = follow.pos;
        }

        let move_pos = target_movable.pos + relative_pos;

        // println!(
        //     "{:?} following {:?} at {:?}",
        //     entity, follow.target, move_pos
        // );

        move_to_commands
            .borrow_mut()
            .insert(entity, MoveCommand { to: move_pos })
            .unwrap();
    }

    Ok(())
}

fn patrol_system(world: &mut World) -> GameResult<()> {
    let entities = &world.entities();
    let patrols = &mut world.write_storage::<PatrolCommand>();
    let move_commands = &mut world.write_storage::<MoveCommand>();

    // patrol
    for (entity, patrol) in (entities, patrols).join() {
        if move_commands.get(entity).is_some() {
            continue;
        }

        let pos = patrol.next();
        // println!("{:?} next pos {:?}", entity, pos);
        move_commands
            .insert(entity, MoveCommand { to: pos })
            .unwrap();
    }

    Ok(())
}

fn move_to_system(world: &mut World) -> GameResult<()> {
    let entities = &world.entities();
    let move_commands = &mut world.write_storage::<MoveCommand>();
    let movables = &mut world.write_storage::<Moveable>();
    let cfg = &world.read_resource::<Cfg>();

    // move to position
    for (entity, movable) in (entities, movables).join() {
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
            move_commands.remove(entity).unwrap();
        } else {
            let dir = delta.normalize();
            let speed = movable.max_speed.min(distance * cfg.speed_reduction);

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
        patrol_system(&mut self.world)?;
        follow_system(&mut self.world)?;
        move_to_system(&mut self.world)?;
        movable_system(delta, &mut self.world)?;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        let entities = self.world.entities();
        let models = &self.world.read_storage::<Model>();
        let movables = &self.world.read_storage::<Moveable>();

        for (e, model, mov) in (&*entities, models, movables).join() {
            // println!("{:?} drawing {:?} at {:?}", e, model, mov);
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                mov.pos,
                model.size,
                0.1,
                model.color,
            )?;
            graphics::draw(ctx, &circle, graphics::DrawParam::default())?;
        }

        let cfg = &self.world.read_resource::<Cfg>();
        let text = graphics::Text::new(format!("{:?}", cfg.deref()));
        graphics::draw(ctx, &text, (cgmath::Point2::new(0.0, 0.0), graphics::WHITE))?;

        graphics::present(ctx)
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: f32, y: f32) {
        self.world.write_resource::<Cfg>().speed_reduction += y * 0.1;
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

fn rotate_vector(dir: Vector2<f32>, point: Vector2<f32>) -> Vector2<f32> {
    let angle = dir.y.atan2(dir.x);

    let qt = Quaternion::from(Euler {
        x: Rad(0.0),
        y: Rad(0.0),
        z: Rad(angle),
    });

    let pointv3 = vec3(point.x, point.y, 0.0);
    let rotated = qt * pointv3;
    vec2(rotated.x, rotated.y)
}

#[cfg(test)]
mod test {
    use super::*;
    use cgmath::{Rad, Vector2};

    macro_rules! assert_delta {
        ($x:expr, $y:expr, $d:expr) => {
            if !($x - $y < $d || $y - $x < $d) {
                panic!();
            }
        };
    }

    #[test]
    fn test_rotate_vector3() {
        let v1 = vec3(1.0, 0.0, 0.0);
        let qt = Quaternion::from(Euler {
            x: Deg(0.0),
            y: Deg(90.0),
            z: Deg(0.0),
        });
        let vf = qt * v1;
        assert!(vf.z == -1.0);
    }

    #[test]
    fn test_vector_angle() {
        let v1: Vector2<f32> = vec2(0.5, 0.5);
        let angle: f32 = Deg::from(Rad(v1.y.atan2(v1.x))).0;
        assert_eq!(angle, 45.0);

        let v1: Vector2<f32> = vec2(-0.5, 0.5);
        let angle: f32 = Deg::from(Rad(v1.y.atan2(v1.x))).0;
        assert_eq!(angle, 135.0);

        let v1: Vector2<f32> = vec2(0.5, -0.5);
        let angle: f32 = Deg::from(Rad(v1.y.atan2(v1.x))).0;
        assert_eq!(angle, -45.0);
    }

    #[test]
    fn test_rotate_vector() {
        let point = vec2(0.0, 1.0);

        let dir = vec2(1.0, 0.0);
        let rotated = rotate_vector(dir, point);
        assert_eq!(rotated, vec2(0.0, 1.0));

        let dir = vec2(-1.0, 0.0);
        let rotated = rotate_vector(dir, point);
        assert_delta!(rotated.x, 0.0, 0.001);
        assert_delta!(rotated.y, -1.0, 0.001);
    }
}
