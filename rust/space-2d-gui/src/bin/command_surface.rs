use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler};
use ggez::graphics::Color;
use ggez::{graphics, timer, Context, ContextBuilder, GameError, GameResult};
use nalgebra::{Point2, Rotation2, Vector2};
use rand::{thread_rng, Rng};
use specs::prelude::*;
use specs::{World, WorldExt};
use specs_derive::Component;
use std::borrow::BorrowMut;

type P3 = nalgebra::Point3<f32>;
type P2 = nalgebra::Point2<f32>;
type V3 = nalgebra::Vector3<f32>;
type V2 = nalgebra::Vector2<f32>;

const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 600;

struct App {
    world: World,
    image: graphics::Image,
}

#[derive(Debug, Clone, Component)]
struct MapCfg {}

#[derive(Debug, Clone, Copy)]
enum ObjKind {
    Base,
    Land,
    Air,
}

#[derive(Debug, Clone, Component)]
enum Action {
    None,
    MoveTo(P2),
}

#[derive(Debug, Clone, Component)]
struct MObj {
    pos: P3,
    speed: V3,
    kind: ObjKind,
}

fn create_entity(world: &mut World, kind: ObjKind, coords: [f32; 3]) -> Entity {
    world
        .create_entity()
        .with(MObj {
            pos: coords.into(),
            speed: [0.0, 0.0, 0.0].into(),
            kind,
        })
        .build()
}

impl App {
    pub fn new(ctx: &mut Context) -> GameResult<App> {
        // create world
        let mut world = World::new();
        world.register::<MapCfg>();
        world.register::<MObj>();
        world.register::<Action>();

        create_entity(&mut world, ObjKind::Base, [150.0, 400.0, 0.0]);
        create_entity(&mut world, ObjKind::Base, [650.0, 300.0, 0.0]);
        let e_air = create_entity(&mut world, ObjKind::Air, [650.0, 300.0, 0.0]);
        world
            .write_storage::<Action>()
            .borrow_mut()
            .insert(e_air, Action::MoveTo([150.0, 400.0].into()));

        ggez::filesystem::print_all(ctx);

        // 959x476
        let image = graphics::Image::new(ctx, "/mars_map.png")?;

        let app = App { world, image };
        Ok(app)
    }
}

impl EventHandler<GameError> for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let time = ggez::timer::delta(ctx).as_secs_f32();

        let mut actions = self.world.write_storage::<Action>();
        let mut objs = self.world.write_storage::<MObj>();

        for (action, obj) in (&mut actions, &mut objs).join() {
            match action {
                Action::None => {}
                Action::MoveTo(to) => {
                    let speed = 10.0;
                    let delta: V3 = P3::new(to.x, to.y, 0.0) - &obj.pos;
                    let normalized = delta.normalize();
                    obj.pos += normalized * time * speed;
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
        let scale: [f32; 2] = [
            SCREEN_W as f32 / self.image.width() as f32,
            SCREEN_H as f32 / self.image.height() as f32,
        ];

        graphics::draw(
            ctx,
            &self.image,
            graphics::DrawParam::default().scale::<[f32; 2]>(scale.into()),
        )?;

        {
            let objects = self.world.read_storage::<MObj>();

            let air_mesh = graphics::MeshBuilder::new()
                .circle::<[f32; 2]>(
                    graphics::DrawMode::stroke(1.0),
                    [0.0, 0.0],
                    5.0,
                    1.0,
                    Color::BLUE,
                )?
                .build(ctx)?;

            let base_mesh = graphics::MeshBuilder::new()
                .polygon::<[f32; 2]>(
                    graphics::DrawMode::stroke(1.0),
                    &[[-10.0, -5.0], [10.0, -5.0], [10.0, 5.0], [-10.0, 5.0]],
                    graphics::Color::BLUE,
                )?
                .line(&[[-10.0, -5.0], [10.0, 5.0]], 1.0, Color::BLUE)?
                .line(&[[-10.0, 5.0], [10.0, -5.0]], 1.0, Color::BLUE)?
                .build(ctx)?;

            for o in (&objects).join() {
                let mesh = match o.kind {
                    ObjKind::Base => &base_mesh,
                    ObjKind::Land => &air_mesh,
                    ObjKind::Air => &air_mesh,
                };

                graphics::draw(ctx, mesh, ([o.pos.x, o.pos.y], 0.0, Color::WHITE))?;
            }
        }

        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }
}

fn main() {
    let resource_dir = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = std::path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        std::path::PathBuf::from("./resources")
    };

    // Make a Context.
    let mut window_mode: WindowMode = Default::default();
    window_mode.resizable = true;
    window_mode.width = SCREEN_W as f32;
    window_mode.height = SCREEN_H as f32;

    let (mut ctx, event_loop) = ContextBuilder::new("my_game", "Cool Game Author")
        .window_mode(window_mode)
        .add_resource_path(resource_dir)
        .build()
        .expect("aieee, could not create ggez context!");

    let app = App::new(&mut ctx).unwrap();

    // Run!
    event::run(ctx, event_loop, app);
}
