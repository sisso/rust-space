use commons::math::{rotate_vector_by_angle, P2, V2};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler};
use ggez::glam::Vec2;
use ggez::graphics::{
    self, Canvas, Color, DrawMode, DrawParam, FillOptions, Mesh, MeshBuilder, StrokeOptions,
};
use ggez::{Context, ContextBuilder, GameResult};
use itertools::Itertools;
use log::LevelFilter;
use std::collections::VecDeque;

const MAX_TRACE_POINTS: usize = 1000;

pub struct Planet {
    pos: P2,
    distance: f32,
    orbit_speed: f32,
    starting_angle: f32,
    trace: VecDeque<P2>,
}

// trait FleetModelImpl {}
//
// type FleetModel = Box<dyn FleetModelImpl>;

enum FleetModel {
    Direct,
    Predict,
}

struct Fleet {
    pos: P2,
    target_planet: usize,
    trace: VecDeque<P2>,
    model: FleetModel,
    speed: f32,
    color: Color,
}

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .filter(Some("newtonian_ships"), LevelFilter::Debug)
        .init();

    // Make a Context.
    let (mut ctx, event_loop) = ContextBuilder::new("newton", "nobody")
        .window_mode(
            WindowMode::default()
                .dimensions(1900.0, 1400.0)
                .resizable(true),
        )
        .build()
        .expect("fail");

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let app = App::new(&mut ctx);

    // Run!
    event::run(ctx, event_loop, app);
}

struct App {
    planets: Vec<Planet>,
    fleets: Vec<Fleet>,
    time_mult: f32,
}

impl App {
    pub fn new(ctx: &mut Context) -> App {
        App {
            planets: vec![
                Planet {
                    pos: Default::default(),
                    distance: 100.0,
                    orbit_speed: 0.1,
                    starting_angle: 0.0,
                    trace: Default::default(),
                },
                Planet {
                    pos: Default::default(),
                    distance: 300.0,
                    orbit_speed: 0.05,
                    starting_angle: std::f32::consts::PI,
                    trace: Default::default(),
                },
            ],
            fleets: vec![
                Fleet {
                    pos: Default::default(),
                    target_planet: 0,
                    trace: Default::default(),
                    model: FleetModel::Direct,
                    speed: 20.0,
                    color: Color::RED,
                },
                Fleet {
                    pos: Default::default(),
                    target_planet: 0,
                    trace: Default::default(),
                    model: FleetModel::Predict,
                    speed: 20.0,
                    color: Color::GREEN,
                },
            ],
            time_mult: 1.0,
        }
    }

    fn draw_orbit(
        &mut self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        camera_pos: Vec2,
    ) -> GameResult {
        for p in &self.planets {
            let circle = Mesh::from_data(
                ctx,
                MeshBuilder::new()
                    .circle(
                        DrawMode::Stroke(StrokeOptions::default()),
                        camera_pos,
                        p.distance,
                        1.0,
                        Color::new(0.4, 0.4, 0.4, 1.0),
                    )?
                    .build(),
            );
            canvas.draw(&circle, DrawParam::default());
        }
        Ok(())
    }

    fn draw_trace(
        ctx: &mut Context,
        canvas: &mut Canvas,
        camera_pos: Vec2,
        trace: &VecDeque<P2>,
        color: Color,
    ) -> GameResult {
        if trace.len() > 2 {
            let mut mb = MeshBuilder::new();
            let points: Vec<P2> = trace
                .iter()
                .map(|p| *p + camera_pos)
                .tuple_windows()
                .flat_map(|(a, b)| vec![a, b])
                .collect::<Vec<_>>();
            mb.line(&points, 1.0, color)?;
            let mesh = Mesh::from_data(ctx, mb.build());
            canvas.draw(&mesh, DrawParam::default());
        }
        Ok(())
    }
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let time = ctx.time.time_since_start().as_secs_f32();
        let delta_time = ctx.time.delta().as_secs_f32();

        // update planets
        for planet in &mut self.planets {
            planet.pos = predict_planet_position(planet, time);
            planet.trace.push_back(planet.pos);
            while planet.trace.len() > MAX_TRACE_POINTS {
                planet.trace.pop_front();
            }
        }

        // update fleets
        for fleet in &mut self.fleets {
            match fleet.model {
                FleetModel::Direct => {
                    let target_pos = &self.planets[fleet.target_planet].pos;
                    let delta = *target_pos - fleet.pos;
                    let distance = delta.length();

                    if distance < 1.0 {
                        log::debug!("fleet arrive, next target");
                        fleet.target_planet =
                            get_next_target(self.planets.len(), fleet.target_planet);
                    } else {
                        let change = delta.normalize() * fleet.speed * delta_time;
                        let new_pos = fleet.pos + change;
                        log::trace!(
                            "fleet at {:?} moving by {:?}, new pos {:?}",
                            fleet.pos,
                            change,
                            new_pos
                        );
                        fleet.pos = new_pos;
                    }

                    fleet.trace.push_back(fleet.pos);
                    while fleet.trace.len() > MAX_TRACE_POINTS {
                        fleet.trace.pop_front();
                    }
                }
                FleetModel::Predict => {
                    let target = &self.planets[fleet.target_planet];
                    let target_pos = target.pos;
                    let delta = target_pos - fleet.pos;
                    let distance = delta.length();

                    if distance < 1.0 {
                        log::debug!("fleet arrive, next target");
                        fleet.target_planet =
                            get_next_target(self.planets.len(), fleet.target_planet);
                    } else {
                        // first interaction
                        let time_to_target = distance / fleet.speed;

                        let predicted_target_pos =
                            predict_planet_position(target, time + time_to_target);
                        let delta = predicted_target_pos - fleet.pos;
                        let change = delta.normalize() * fleet.speed * delta_time;
                        let new_pos = fleet.pos + change;
                        log::trace!(
                            "fleet at {:?} moving by {:?}, new pos {:?}",
                            fleet.pos,
                            change,
                            new_pos
                        );
                        fleet.pos = new_pos;
                    }

                    fleet.trace.push_back(fleet.pos);
                    while fleet.trace.len() > MAX_TRACE_POINTS {
                        fleet.trace.pop_front();
                    }
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);

        let (width, height) = ctx.gfx.size();

        let camera_pos = V2::new(width, height) * 0.5;

        let planet_mesh = Mesh::from_data(
            ctx,
            MeshBuilder::new()
                .circle(DrawMode::stroke(1.0), P2::ZERO, 10.0, 1.0, Color::WHITE)?
                .build(),
        );

        // draw sum
        canvas.draw(
            &planet_mesh,
            DrawParam::new().dest(camera_pos).color(Color::YELLOW),
        );

        // draw planets
        for planet in &self.planets {
            canvas.draw(&planet_mesh, DrawParam::new().dest(planet.pos + camera_pos));

            // draw trace
            Self::draw_trace(
                ctx,
                &mut canvas,
                camera_pos,
                &planet.trace,
                Color::new(0.3, 0.3, 0.3, 1.0),
            )?;
        }

        // draw orbits
        // self.draw_orbit(ctx, &mut canvas, camera_pos)?;
        let fleet_mesh = Mesh::from_data(
            ctx,
            MeshBuilder::new()
                .circle(DrawMode::fill(), P2::ZERO, 2.5, 1.0, Color::WHITE)?
                .build(),
        );

        for fleet in &self.fleets {
            canvas.draw(
                &fleet_mesh,
                DrawParam::default()
                    .dest(fleet.pos + camera_pos)
                    .color(fleet.color),
            );
            // draw trace
            Self::draw_trace(ctx, &mut canvas, camera_pos, &fleet.trace, fleet.color)?;
        }

        canvas.finish(ctx)
    }
}

fn predict_planet_position(planet: &Planet, time: f32) -> P2 {
    let angle = planet.orbit_speed * time + planet.starting_angle;
    return rotate_vector_by_angle(V2::new(planet.distance, 0.0), angle);
}

fn get_next_target(amount: usize, current: usize) -> usize {
    if current + 1 < amount {
        current + 1
    } else {
        0
    }
}
