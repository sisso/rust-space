use cgmath::{prelude::*, vec2, vec3, Deg, Euler, Quaternion, Rad, Vector2};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler};
use ggez::graphics::Color;
use ggez::{graphics, timer, Context, ContextBuilder, GameResult};
use specs::prelude::*;
use specs::{World, WorldExt};
use specs_derive::Component;
use std::borrow::{Borrow, BorrowMut};
use std::ops::Deref;

// TODO: docking station

#[derive(Clone, Debug, Component)]
struct Debug {
    lines: Vec<(cgmath::Point2<f32>, cgmath::Point2<f32>, Color)>,
}

impl Debug {
    pub fn new() -> Self {
        Debug { lines: Vec::new() }
    }
}

#[derive(Clone, Debug, Component)]
struct Cfg {
    speed_reduction: f32,
    pause: bool,
    vector_epsilon: f32,
    patrol_arrival: bool,
}

#[derive(Clone, Debug, Component)]
struct Time {
    total_time: f32,
    delta_time: f32,
}

#[derive(Clone, Debug, Component)]
struct Station {
    pos: cgmath::Point2<f32>,
    entrance_dir: cgmath::Vector2<f32>,
}

#[derive(Clone, Debug, Component)]
struct Movable {
    pos: cgmath::Point2<f32>,
    max_speed: f32,
    vel: cgmath::Vector2<f32>,
    desired_vel: cgmath::Vector2<f32>,
    max_acc: f32,
    follower_behind_max_speed: Option<f32>,
}

impl Movable {
    fn new(pos: cgmath::Point2<f32>, max_speed: f32, max_acc: f32) -> Self {
        Movable {
            pos,
            max_speed,
            vel: Vector2::zero(),
            desired_vel: Vector2::zero(),
            max_acc,
            follower_behind_max_speed: None,
        }
    }
}

#[derive(Clone, Debug, Component)]
struct MoveCommand {
    to: cgmath::Point2<f32>,
    arrival: bool,
    predict: bool,
}

#[derive(Clone, Debug, Component)]
struct PatrolCommand {
    pub index: usize,
    pub route: Vec<cgmath::Point2<f32>>,
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

    pub fn route_from_current(&self) -> Vec<cgmath::Point2<f32>> {
        let mut r = vec![];
        for i in self.index..self.route.len() {
            r.push(self.route[i]);
        }
        for i in 0..self.index {
            r.push(self.route[i]);
        }
        r.push(r[0]);
        r
    }
}

#[derive(Clone, Debug, Component)]
struct FollowCommand {
    target: Entity,
    /// relative position
    pos: cgmath::Vector2<f32>,
    // current_pos: cgmath::Vector2<f32>,
}

impl FollowCommand {
    pub fn new(target: Entity, pos: Vector2<f32>) -> Self {
        FollowCommand {
            target,
            pos,
            // current_pos: cgmath::vec2(0.0, 0.0),
        }
    }
}

#[derive(Clone, Debug, Component)]
struct Model {
    pos: cgmath::Point2<f32>,
    size: f32,
    color: graphics::Color,
}

impl Model {
    pub fn new(size: f32, color: graphics::Color) -> Self {
        Model {
            pos: cgmath::Point2::new(0.0, 0.0),
            size,
            color,
        }
    }
}

#[derive(Clone, Debug, Component)]
struct MovementPrediction {
    points: Vec<(f32, cgmath::Point2<f32>)>,
}

struct App {
    world: World,
}

impl App {
    pub fn new(ctx: &mut Context) -> GameResult<App> {
        // create world
        let mut world = World::new();
        world.register::<Model>();
        world.register::<Movable>();
        world.register::<Station>();
        world.register::<MoveCommand>();
        world.register::<PatrolCommand>();
        world.register::<FollowCommand>();
        world.register::<MovementPrediction>();
        world.register::<Cfg>();
        world.register::<Time>();

        world.insert(Debug::new());

        world.insert(Cfg {
            speed_reduction: 1.5,
            pause: false,
            vector_epsilon: 2.0,
            patrol_arrival: true,
        });

        world.insert(Time {
            total_time: 0.0,
            delta_time: 0.0,
        });

        // add elements
        App::scenery_patrol_and_follow(&mut world);
        // App::scenery_two_stations(&mut world);

        let game = App { world };

        Ok(game)
    }

    fn scenery_two_stations(world: &mut World) {
        let station_0 = world
            .create_entity()
            .with(Model::new(20.0, graphics::Color::new(0.0, 1.0, 0.0, 1.0)))
            .with(Station {
                pos: cgmath::Point2::new(200.0, 300.0),
                entrance_dir: vec2(1.0, 0.0),
            })
            .build();

        let station_1 = world
            .create_entity()
            .with(Model::new(20.0, graphics::Color::new(1.0, 0.0, 0.0, 1.0)))
            .with(Station {
                pos: cgmath::Point2::new(700.0, 300.0),
                entrance_dir: vec2(0.0, 1.0),
            })
            .build();

        let ship_0 = world
            .create_entity()
            .with(Model::new(2.0, graphics::WHITE))
            .with(Movable::new(cgmath::Point2::new(400.0, 300.0), 54.0, 54.0))
            .build();
    }

    fn scenery_patrol_and_follow(world: &mut World) {
        {
            let entity_0 = world
                .create_entity()
                .with(Model::new(6.0, graphics::WHITE))
                .with(Movable::new(cgmath::Point2::new(400.0, 300.0), 80.0, 70.0))
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
                .with(Model::new(2.0, graphics::Color::new(1.0, 0.0, 0.0, 1.0)))
                .with(Movable::new(cgmath::Point2::new(450.0, 320.0), 70.0, 40.0))
                .with(FollowCommand {
                    target: entity_0,
                    pos: vec2(0.0, -10.0),
                })
                .build();

            world
                .create_entity()
                .with(Model::new(2.0, graphics::Color::new(0.0, 1.0, 0.0, 1.0)))
                .with(Movable::new(cgmath::Point2::new(450.0, 320.0), 55.0, 80.0))
                .with(FollowCommand {
                    target: entity_0,
                    pos: vec2(0.0, 10.0),
                })
                .build();
        }
    }
}

fn follow_command_system(world: &mut World) -> GameResult<()> {
    let entities = world.entities();
    let mut follow_commands = world.write_storage::<FollowCommand>();
    let mut movables = world.write_storage::<Movable>();
    let predictions = &mut world.write_storage::<MovementPrediction>();
    let mut debug = world.write_resource::<Debug>();

    //  collect for each follow the target position
    let mut changes = vec![];
    let mut follower_behind_flag = vec![];

    for (entity, follow, movable) in (&*entities, &follow_commands, &movables).join() {
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

        let target_pos = target_movable.pos + relative_pos;
        let delta_to_pos = target_pos - movable.pos;
        let desired_vel = target_movable.vel + delta_to_pos;

        changes.push((entity, desired_vel));

        predictions.insert(
            entity,
            MovementPrediction {
                points: vec![(0.0, movable.pos), (1.0, target_pos)],
            },
        );

        if delta_to_pos.magnitude() > 10.0 {
            follower_behind_flag.push((follow.target, movable.max_speed));
        }
    }

    for (entity, desired_vel) in changes {
        let movable = movables.get_mut(entity).unwrap();
        movable.desired_vel = desired_vel;
    }

    for (entity, speed) in follower_behind_flag {
        let movable = movables.get_mut(entity).unwrap();

        movable.follower_behind_max_speed = match &movable.follower_behind_max_speed {
            Some(value) if *value > speed => Some(speed),
            Some(value) => Some(*value),
            None => Some(speed),
        };
    }

    Ok(())
}

fn patrol_system(world: &mut World) -> GameResult<()> {
    let entities = world.entities();
    let cfg = world.read_resource::<Cfg>();
    let mut patrols = world.write_storage::<PatrolCommand>();
    let mut move_commands = world.write_storage::<MoveCommand>();
    let mut predictions = world.write_storage::<MovementPrediction>();

    let mut commands = vec![];

    // patrol
    for (entity, patrol, _) in (&*entities, &mut patrols, !&move_commands).join() {
        let pos = patrol.next();
        // println!("{:?} next pos {:?}", entity, pos);
        commands.push((
            entity,
            MoveCommand {
                to: pos,
                arrival: cfg.patrol_arrival,
                predict: false,
            },
        ));

        let points = patrol
            .route_from_current()
            .into_iter()
            .map(|pos| (0.0, pos))
            .collect();

        predictions
            .borrow_mut()
            .insert(entity, MovementPrediction { points })
            .unwrap();
    }

    for (entity, command) in commands {
        move_commands.insert(entity, command).unwrap();
    }

    Ok(())
}

fn move_command_system(world: &mut World) -> GameResult<()> {
    let entities = &world.entities();
    let mut move_commands = world.write_storage::<MoveCommand>();
    let mut movables = world.write_storage::<Movable>();
    let mut predictions = world.write_storage::<MovementPrediction>();
    let cfg = &world.read_resource::<Cfg>();
    let total_time = world.read_resource::<Time>().borrow().total_time;
    let debug = &mut world.write_resource::<Debug>();

    let mut completes = vec![];

    // move to position
    for (entity, movable, move_command) in (entities, &mut movables, &mut move_commands).join() {
        let delta = move_command.to - movable.pos;
        let distance = delta.magnitude();
        if distance < cfg.vector_epsilon {
            // println!("{:?} complete", entity);
            movable.desired_vel = vec2(0.0, 0.0);
            completes.push((entity, move_command.predict));
        } else {
            // let mut follower_reduction = false;
            // let mut arrival_reduction = false;
            let dir = delta.normalize();

            // check if a follower is behind and clear the flag
            let max_speed = if let Some(v) = movable.follower_behind_max_speed.take() {
                // follower_reduction = true;
                movable.max_speed.min(v * 0.8)
            } else {
                movable.max_speed
            };

            let speed = if move_command.arrival {
                max_speed.min(distance)
            } else {
                max_speed
            };

            movable.desired_vel = dir * speed;

            if move_command.predict {
                let mut points = vec![];
                points.push((total_time, movable.pos));

                let arrival_time = distance / speed;
                points.push((total_time + arrival_time, move_command.to));

                predictions
                    .insert(entity, MovementPrediction { points: points })
                    .unwrap();
            }
        }
    }

    for (entity, was_predicted) in completes {
        move_commands.remove(entity);
        if was_predicted {
            predictions.remove(entity);
        }
    }

    Ok(())
}

fn movable_system(delta: f32, world: &mut World) -> GameResult<()> {
    let mut movables = world.write_storage::<Movable>();

    for (movable,) in (&mut movables,).join() {
        // println!("{:?} moving at {}", movable, delta);
        let mut delta_vel = movable.desired_vel - movable.vel;
        let acc = delta_vel.normalize() * movable.max_acc;
        movable.vel = movable.vel + acc * delta;
        movable.pos = movable.pos + movable.vel * delta;
    }

    Ok(())
}

fn model_system(world: &mut World) -> GameResult<()> {
    let movables = world.read_storage::<Movable>();
    let stations = world.read_storage::<Station>();
    let mut models = world.write_storage::<Model>();

    for (movable, model) in (&movables, &mut models).join() {
        model.pos = movable.pos;
    }

    for (station, model) in (&stations, &mut models).join() {
        model.pos = station.pos;
    }

    Ok(())
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let delta = timer::delta(ctx).as_secs_f32();

        {
            let time = &mut self.world.write_resource::<Time>();
            time.total_time += delta;
            time.delta_time = delta;
        }

        if !self.world.read_resource::<Cfg>().borrow().pause {
            patrol_system(&mut self.world)?;
            follow_command_system(&mut self.world)?;
            move_command_system(&mut self.world)?;
            movable_system(delta, &mut self.world)?;
        }

        model_system(&mut self.world)?;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        let entities = self.world.entities();
        let models = &self.world.read_storage::<Model>();
        let movables = &self.world.read_storage::<Movable>();
        let predictions = self.world.read_storage::<MovementPrediction>();

        for (e, model, prediction, movable) in
            (&*entities, models, predictions.maybe(), movables).join()
        {
            // println!("{:?} drawing {:?} at {:?}", e, model, mov);
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                model.pos,
                model.size,
                0.1,
                model.color,
            )?;
            graphics::draw(ctx, &circle, graphics::DrawParam::default())?;

            if let Some(prediction) = prediction {
                if prediction.points.len() > 1 {
                    let color = Color::new(0.9, 0.23, 0.1, 0.5);
                    let points: Vec<cgmath::Point2<f32>> = prediction
                        .points
                        .iter()
                        .map(|(time, pos)| pos.clone())
                        .collect();

                    let line_mesh = graphics::Mesh::new_line(ctx, points.as_slice(), 1.0, color)?;
                    graphics::draw(ctx, &line_mesh, graphics::DrawParam::default())?;
                }
            }

            if movable.vel.magnitude2() > 1.0 {
                let color = Color::new(0.0, 0.0, 0.9, 0.25);
                let points: Vec<cgmath::Point2<f32>> = vec![movable.pos, movable.pos + movable.vel];
                let line_mesh = graphics::Mesh::new_line(ctx, points.as_slice(), 1.0, color)?;
                graphics::draw(ctx, &line_mesh, graphics::DrawParam::default())?;
            }

            if movable.desired_vel.magnitude2() > 1.0 {
                let color = Color::new(0.0, 0.9, 0.0, 0.25);
                let points: Vec<cgmath::Point2<f32>> =
                    vec![movable.pos, movable.pos + movable.desired_vel];
                let line_mesh = graphics::Mesh::new_line(ctx, points.as_slice(), 1.0, color)?;
                graphics::draw(ctx, &line_mesh, graphics::DrawParam::default())?;
            }
        }

        let debug = &mut self.world.write_resource::<Debug>();
        for (a, b, color) in std::mem::replace(&mut debug.lines, Vec::new()) {
            let points: Vec<cgmath::Point2<f32>> = vec![a, b];
            let line_mesh = graphics::Mesh::new_line(ctx, points.as_slice(), 1.0, color)?;
            graphics::draw(ctx, &line_mesh, graphics::DrawParam::default())?;
        }

        let cfg = &self.world.read_resource::<Cfg>();
        let text = graphics::Text::new(format!("{:?}", cfg.deref()));
        graphics::draw(ctx, &text, (cgmath::Point2::new(0.0, 0.0), graphics::WHITE))?;

        graphics::present(ctx)
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: f32, y: f32) {
        self.world.write_resource::<Cfg>().speed_reduction += y * 0.1;
    }

    fn text_input_event(&mut self, _ctx: &mut Context, character: char) {
        match character {
            ' ' => {
                let cfg = &mut self.world.write_resource::<Cfg>();
                cfg.pause = !cfg.pause;
            }

            'a' => {
                let cfg = &mut self.world.write_resource::<Cfg>();
                cfg.patrol_arrival = !cfg.patrol_arrival;
            }

            _ => {}
        }
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
