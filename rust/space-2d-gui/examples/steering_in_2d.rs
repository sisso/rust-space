use approx::assert_relative_eq;
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler};
use ggez::graphics::{Color, StrokeOptions};
use ggez::{graphics, timer, Context, ContextBuilder, GameResult};
use nalgebra::{self as na, Point2, Rotation2, Similarity2, Vector2};
use rand::{thread_rng, Rng};
use specs::prelude::*;
use specs::{World, WorldExt};
use specs_derive::Component;
use std::borrow::{Borrow, BorrowMut};
use std::ops::Deref;

// TODO: replace cgmath by nalge

type V2 = Vector2<f32>;
type P2 = Point2<f32>;

const ARRIVAL_DISTANCE: f32 = 2.0;

#[derive(Clone, Debug, Component)]
struct Debug {
    lines: Vec<(P2, P2, Color)>,
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
pub struct Station {
    pos: P2,
    entrance_dir: V2,
    entrance_distance: f32,
}

impl Station {
    pub fn new(pos: P2, entrance_dir: V2, entrance_distance: f32) -> Self {
        Station {
            pos,
            entrance_dir,
            entrance_distance,
        }
    }

    pub fn get_entrance_pos(&self) -> P2 {
        self.pos + self.entrance_dir * self.entrance_distance
    }
}

#[derive(Clone, Debug, Component)]
struct Movable {
    pos: P2,
    max_speed: f32,
    vel: V2,
    desired_vel: V2,
    max_acc: f32,
    follower_behind_max_speed: Option<f32>,
}

impl Movable {
    fn new(pos: P2, max_speed: f32, max_acc: f32) -> Self {
        Movable {
            pos,
            max_speed,
            vel: V2::new(0.0, 0.0),
            desired_vel: V2::new(0.0, 0.0),
            max_acc,
            follower_behind_max_speed: None,
        }
    }

    /// max speed considering followers
    fn get_max_speed(&self) -> f32 {
        self.follower_behind_max_speed.unwrap_or(self.max_speed)
    }
}

#[derive(Clone, Debug, Component)]
struct MoveCommand {
    to: P2,
    arrival: bool,
    predict: bool,
}

#[derive(Clone, Debug, Component)]
struct PatrolCommand {
    pub index: usize,
    pub route: Vec<P2>,
}

impl PatrolCommand {
    pub fn current(&self) -> P2 {
        self.route[self.index]
    }

    /// move to next point
    pub fn next(&mut self) {
        self.index += 1;
        if self.index >= self.route.len() {
            self.index = 0;
        }
    }

    pub fn route_from_next(&self) -> Vec<P2> {
        let mut r = vec![];
        for i in self.index..self.route.len() {
            r.push(self.route[i]);
        }
        for i in 0..self.index {
            r.push(self.route[i]);
        }
        r
    }
}

#[derive(Clone, Debug, Component)]
struct FollowCommand {
    target: Entity,
    /// relative position
    relative_pos: P2,
}

impl FollowCommand {
    pub fn new(target: Entity, pos: P2) -> Self {
        FollowCommand {
            target,
            relative_pos: pos,
        }
    }
}

#[derive(Clone, Debug)]
pub enum TradeCommandState {
    MoveToDock,
    Docking,
    Docked { complete_time: f32 },
    Undocking,
}

#[derive(Clone, Debug, Component)]
pub struct TradeCommand {
    stations: Vec<Entity>,
    index: usize,
    state: TradeCommandState,
}

impl TradeCommand {
    pub fn new(stations: Vec<Entity>) -> Self {
        TradeCommand {
            stations,
            index: 0,
            state: TradeCommandState::MoveToDock,
        }
    }

    pub fn current(&self) -> Entity {
        self.stations[self.index]
    }

    pub fn next(&mut self) {
        self.index += 1;
        if self.index >= self.stations.len() {
            self.index = 0;
        }
    }
}

#[derive(Clone, Debug, Component)]
struct Model {
    pos: P2,
    size: f32,
    color: graphics::Color,
}

impl Model {
    pub fn new(size: f32, color: graphics::Color) -> Self {
        Model {
            pos: Point2::new(0.0, 0.0),
            size,
            color,
        }
    }
}

#[derive(Clone, Debug, Component)]
struct MovementPrediction {
    points: Vec<P2>,
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
        let mut relative_pos =
            rotate_vector(target_movable.vel.normalize(), follow.relative_pos.clone());
        if relative_pos.x.is_nan() || relative_pos.y.is_nan() {
            relative_pos = follow.relative_pos.clone();
        }

        let target_pos = target_movable.pos.clone() + relative_pos.coords;
        let delta_to_pos = target_pos - movable.pos.clone();
        let desired_vel = target_movable.vel.clone() + delta_to_pos;

        changes.push((entity, desired_vel));

        predictions
            .insert(
                entity,
                MovementPrediction {
                    points: vec![movable.pos, target_pos],
                },
            )
            .unwrap();

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

fn patrol_command_system(world: &mut World) -> GameResult<()> {
    let entities = world.entities();
    let cfg = world.read_resource::<Cfg>();
    let mut patrols = world.write_storage::<PatrolCommand>();
    let mut predictions = world.write_storage::<MovementPrediction>();
    let mut movable = world.write_storage::<Movable>();

    // patrol
    for (entity, command, movable) in (&*entities, &mut patrols, &mut movable).join() {
        let mut result = action_move_to(
            movable.pos,
            command.current(),
            movable.get_max_speed(),
            false,
        );

        let is_complete = result.complete;

        // if we complete, get path for next step
        if result.complete {
            command.next();

            result = action_move_to(
                movable.pos,
                command.current(),
                movable.get_max_speed(),
                false,
            );
        }

        // update movement
        movable.desired_vel = result.desired_vel;

        // update prediction
        let prediction = predictions.borrow_mut().get_mut(entity);
        if is_complete || prediction.is_none() {
            let mut points = vec![];
            points.push(movable.pos);
            points.push(command.current());
            points.extend(command.route_from_next());

            predictions
                .borrow_mut()
                .insert(entity, MovementPrediction { points })
                .unwrap();
        } else {
            prediction.unwrap().points[0] = movable.pos;
        }
    }

    Ok(())
}

fn move_command_system(world: &mut World) -> GameResult<()> {
    let entities = &world.entities();
    let mut move_commands = world.write_storage::<MoveCommand>();
    let mut movables = world.write_storage::<Movable>();
    let mut predictions = world.write_storage::<MovementPrediction>();
    let mut completes = vec![];

    // move to position
    for (entity, movable, move_command) in (entities, &mut movables, &mut move_commands).join() {
        // check if a follower is behind and clear the flag
        let max_speed = if let Some(v) = movable.follower_behind_max_speed.take() {
            // follower_reduction = true;
            movable.max_speed.min(v * 0.8)
        } else {
            movable.max_speed
        };

        let result = action_move_to(
            movable.pos,
            move_command.to,
            max_speed,
            move_command.arrival,
        );
        movable.desired_vel = result.desired_vel;

        if result.complete {
            completes.push((entity, move_command.predict));
        } else {
            if move_command.predict {
                let mut points = vec![];
                points.push(movable.pos);
                points.push(move_command.to);

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

struct ActionMoveResult {
    desired_vel: V2,
    complete: bool,
}

fn action_move_to(
    current_pos: P2,
    target_pos: P2,
    max_speed: f32,
    arrival: bool,
) -> ActionMoveResult {
    let delta = target_pos - current_pos;
    let distance = delta.magnitude();

    if distance < ARRIVAL_DISTANCE {
        ActionMoveResult {
            desired_vel: V2::new(0.0, 0.0),
            complete: true,
        }
    } else {
        let dir = delta / distance;
        let speed = if arrival {
            max_speed.min(distance)
        } else {
            max_speed
        };

        let desired_vel = dir * speed;
        ActionMoveResult {
            desired_vel,
            complete: false,
        }
    }
}

fn movable_system(delta: f32, world: &mut World) -> GameResult<()> {
    let mut movables = world.write_storage::<Movable>();

    for (movable,) in (&mut movables,).join() {
        tick_movable(delta, movable);
    }

    Ok(())
}

fn tick_movable(delta: f32, movable: &mut Movable) {
    let delta_vel = movable.desired_vel.clone() - movable.vel.clone();
    let mag = delta_vel.magnitude();
    if mag > 0.01 {
        let acc = delta_vel / mag * movable.max_acc;
        movable.vel = movable.vel.clone() + acc * delta;
    }
    movable.pos = movable.pos.clone() + movable.vel.clone() * delta;
}

fn trade_command_system(world: &mut World) -> GameResult<()> {
    let entities = &world.entities();
    let mut trade_commands = world.write_storage::<TradeCommand>();
    let mut movables = world.write_storage::<Movable>();
    let mut predictions = world.write_storage::<MovementPrediction>();
    let stations = &world.read_storage::<Station>();
    let total_time = world.read_resource::<Time>().total_time;

    for (entity, command, movable, prediction) in (
        *&entities,
        &mut trade_commands,
        &mut movables,
        predictions.maybe(),
    )
        .join()
    {
        let station = stations.get(command.current()).unwrap();

        match command.state {
            TradeCommandState::MoveToDock => {
                let entrance_pos = station.get_entrance_pos();

                let result =
                    action_move_to(movable.pos, entrance_pos, movable.get_max_speed(), true);

                movable.desired_vel = result.desired_vel;
                if result.complete {
                    command.state = TradeCommandState::Docking;
                }
            }

            TradeCommandState::Docking => {
                let result =
                    action_move_to(movable.pos, station.pos, movable.get_max_speed(), true);

                movable.desired_vel = result.desired_vel;
                if result.complete {
                    command.state = TradeCommandState::Docked {
                        complete_time: total_time + 1.0,
                    };
                }
            }

            TradeCommandState::Docked { complete_time } if total_time > complete_time => {
                command.state = TradeCommandState::Undocking;
            }

            TradeCommandState::Docked { .. } => {}

            TradeCommandState::Undocking => {
                let entrance_pos = station.get_entrance_pos();

                let result =
                    action_move_to(movable.pos, entrance_pos, movable.get_max_speed(), false);

                movable.desired_vel = result.desired_vel;
                if result.complete {
                    command.next();
                    command.state = TradeCommandState::MoveToDock;
                }
            }
        }
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
        world.register::<TradeCommand>();

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
        App::scenery_two_stations(&mut world);

        let game = App { world };

        Ok(game)
    }

    fn scenery_patrol_and_follow(world: &mut World) {
        {
            let entity_0 = world
                .create_entity()
                .with(Model::new(6.0, graphics::WHITE))
                .with(Movable::new(Point2::new(400.0, 300.0), 80.0, 70.0))
                .with(PatrolCommand {
                    index: 0,
                    route: vec![
                        Point2::new(200.0, 300.0),
                        Point2::new(400.0, 150.0),
                        Point2::new(600.0, 300.0),
                        Point2::new(400.0, 550.0),
                    ],
                })
                .build();

            world
                .create_entity()
                .with(Model::new(2.0, graphics::Color::new(1.0, 0.0, 0.0, 1.0)))
                .with(Movable::new(Point2::new(450.0, 320.0), 70.0, 40.0))
                .with(FollowCommand {
                    target: entity_0,
                    relative_pos: P2::new(0.0, -10.0),
                })
                .build();

            world
                .create_entity()
                .with(Model::new(2.0, graphics::Color::new(0.0, 1.0, 0.0, 1.0)))
                .with(Movable::new(Point2::new(450.0, 320.0), 55.0, 80.0))
                .with(FollowCommand {
                    target: entity_0,
                    relative_pos: P2::new(0.0, 10.0),
                })
                .build();
        }
    }

    fn scenery_two_stations(world: &mut World) {
        let station_0 = world
            .create_entity()
            .with(Model::new(15.0, graphics::Color::new(0.0, 1.0, 0.0, 1.0)))
            .with(Station {
                pos: Point2::new(110.0, 200.0),
                entrance_dir: V2::new(1.0, 0.0),
                entrance_distance: 40.0,
            })
            .build();

        let station_1 = world
            .create_entity()
            .with(Model::new(15.0, graphics::Color::new(1.0, 0.0, 0.0, 1.0)))
            .with(Station {
                pos: Point2::new(700.0, 300.0),
                entrance_dir: V2::new(0.0, 1.0),
                entrance_distance: 40.0,
            })
            .build();

        let mut rng = thread_rng();

        for _ in 0..10 {
            let x = rng.gen_range(0, 800) as f32;
            let y = rng.gen_range(0, 600) as f32;
            let speed = rng.gen_range(20, 100) as f32;
            let acc = rng.gen_range(50, 100) as f32;

            world
                .create_entity()
                .with(Model::new(4.0, graphics::WHITE))
                .with(Movable::new(Point2::new(x, y), speed, acc))
                .with(TradeCommand::new(vec![station_0, station_1]))
                .build();
        }
    }
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
            patrol_command_system(&mut self.world)?;
            follow_command_system(&mut self.world)?;
            move_command_system(&mut self.world)?;
            movable_system(delta, &mut self.world)?;
            trade_command_system(&mut self.world)?;
        }

        model_system(&mut self.world)?;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        let entities = self.world.entities();
        let models = self.world.read_storage::<Model>();
        let movables = self.world.read_storage::<Movable>();
        let predictions = self.world.read_storage::<MovementPrediction>();
        let stations = self.world.read_storage::<Station>();

        for (e, model, prediction, movable, station) in (
            &*entities,
            &models,
            predictions.maybe(),
            movables.maybe(),
            stations.maybe(),
        )
            .join()
        {
            // println!("drawing {:?}: {:?}", e, model);

            // draw ship
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                model.pos,
                model.size,
                0.1,
                model.color,
            )?;
            graphics::draw(ctx, &circle, graphics::DrawParam::default())?;

            // draw prediction
            if let Some(prediction) = prediction {
                if prediction.points.len() > 1 {
                    let color = Color::new(0.9, 0.23, 0.1, 0.5);
                    let points: Vec<P2> = prediction.points.clone();

                    let line_mesh = graphics::Mesh::new_line(ctx, points.as_slice(), 1.0, color)?;
                    graphics::draw(ctx, &line_mesh, graphics::DrawParam::default())?;
                }
            }

            // draw movements
            if let Some(movable) = movable {
                if movable.vel.magnitude() > 1.0 {
                    let color = Color::new(0.0, 0.0, 0.9, 0.25);
                    let points: Vec<P2> = vec![movable.pos, movable.pos + movable.vel];
                    let line_mesh = graphics::Mesh::new_line(ctx, points.as_slice(), 1.0, color)?;
                    graphics::draw(ctx, &line_mesh, graphics::DrawParam::default())?;
                }

                if movable.desired_vel.magnitude() > 1.0 {
                    let color = Color::new(0.0, 0.9, 0.0, 0.25);
                    let points: Vec<P2> = vec![movable.pos, movable.pos + movable.desired_vel];
                    let line_mesh = graphics::Mesh::new_line(ctx, points.as_slice(), 1.0, color)?;
                    graphics::draw(ctx, &line_mesh, graphics::DrawParam::default())?;
                }
            }

            // draw stations entrance
            if let Some(station) = station {
                let color = Color::new(0.1, 0.5, 1.0, 0.5);
                let entrance_incoming_pos = station.get_entrance_pos();

                let points: Vec<P2> = vec![station.pos, entrance_incoming_pos];
                let line_mesh = graphics::Mesh::new_line(ctx, points.as_slice(), 1.0, color)?;
                graphics::draw(ctx, &line_mesh, graphics::DrawParam::default())?;

                let circle = graphics::Mesh::new_circle(
                    ctx,
                    graphics::DrawMode::Stroke(StrokeOptions::default()),
                    entrance_incoming_pos,
                    10.0,
                    0.1,
                    model.color,
                )?;
                graphics::draw(ctx, &circle, graphics::DrawParam::default())?;
            }
        }

        let debug = &mut self.world.write_resource::<Debug>();
        for (a, b, color) in std::mem::replace(&mut debug.lines, Vec::new()) {
            let points: Vec<P2> = vec![a, b];
            let line_mesh = graphics::Mesh::new_line(ctx, points.as_slice(), 1.0, color)?;
            graphics::draw(ctx, &line_mesh, graphics::DrawParam::default())?;
        }

        let cfg = &self.world.read_resource::<Cfg>();
        let text = graphics::Text::new(format!("{:?}", cfg.deref()));
        graphics::draw(ctx, &text, (Point2::new(0.0, 0.0), graphics::WHITE))?;

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

pub fn angle_vector(v: V2) -> f32 {
    v.y.atan2(v.x)
}

pub fn rotate_vector(dir: V2, point: P2) -> P2 {
    let angle = angle_vector(dir);
    rotate_vector_by_angle(point, angle)
}

pub fn rotate_vector_by_angle(point: P2, angle: f32) -> P2 {
    let rotation = Rotation2::new(angle);
    rotation * point
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! assert_delta {
        ($x:expr, $y:expr, $d:expr) => {
            if !($x - $y < $d || $y - $x < $d) {
                panic!();
            }
        };
    }

    #[test]
    fn test_rotate_vector() {
        let point = P2::new(0.0, 1.0);

        let dir = V2::new(1.0, 0.0);
        let rotated = rotate_vector(dir, point.clone());
        assert_eq!(rotated, P2::new(0.0, 1.0));

        let dir = V2::new(-1.0, 0.0);
        let rotated = rotate_vector(dir, point);
        assert_delta!(rotated.x, 0.0, 0.001);
        assert_delta!(rotated.y, -1.0, 0.001);
    }

    #[test]
    fn test_tick_move_with_empty_movable() {
        let mut movable = Movable::new(P2::new(400.0, 300.0), 10.0, 10.0);
        tick_movable(0.0, &mut movable);
        assert_eq!(movable.pos.x, 400.0);
        assert_eq!(movable.pos.y, 300.0);
    }
}
