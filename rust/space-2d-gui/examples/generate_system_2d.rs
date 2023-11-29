use commons::math::{P2, V2};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, MouseButton};
use ggez::graphics::{self, Color, DrawMode, DrawParam, Rect};
use ggez::input::keyboard::KeyInput;
use ggez::{Context, ContextBuilder, GameError, GameResult};
use rand::prelude::StdRng;
use space_2d_gui::system_generator::*;

const BODY_SIZE: f32 = 0.25;

struct Viewer {
    screen_width: i32,
    screen_height: i32,
    system: System,
    seed: u64,
    center: V2,
    target: P2,
    zoom: f32,
    selected: Option<SelectedObject>,
}

struct SelectedObject {
    index: usize,
    text: ggez::graphics::Text,
}

fn main() {
    let screen_width = 1920;
    let screen_height = 1080;

    let mut window_mode: WindowMode = Default::default();
    window_mode.resizable = true;
    window_mode.width = screen_width as f32;
    window_mode.height = screen_height as f32;
    window_mode.resize_on_scale_factor_change = false;

    let (ctx, event_loop) = ContextBuilder::new("my_game", "Cool Game Author")
        .window_mode(window_mode)
        .build()
        .expect("aieee, could not create ggez context!");

    let system = load_system(0);
    let middle_x = screen_width as f32 / 2.0;
    let middle_y = screen_height as f32 / 2.0;
    let origin = V2::new(middle_x, middle_y);

    let viewer = Viewer {
        screen_width,
        screen_height,
        system,
        seed: 0,
        center: origin,
        target: P2::new(0.0, 0.0),
        zoom: 1.0,
        selected: None,
    };

    event::run(ctx, event_loop, viewer);
}

fn load_system(seed: u64) -> System {
    let mut rng: StdRng = rand::SeedableRng::seed_from_u64(seed);
    let cfg = space_2d_gui::system_generator::new_config(
        std::path::PathBuf::from("space-2d-gui/data").as_path(),
    );
    let system = new_system(&cfg, &mut rng);
    system
}

impl Viewer {
    fn screen_to_world(&self, pos: P2) -> P2 {
        (pos - self.center) / self.zoom
    }

    fn world_to_screen(&self, p2: P2) -> P2 {
        p2 * self.zoom + self.center
    }

    fn compute_local_pos(&self, index: usize, total_time: f32) -> P2 {
        let bodies = &self.system.bodies;

        if index == 0 {
            P2::new(0.0, 0.0)
        } else {
            let relative = commons::math::rotate_vector_by_angle(
                P2::new(1.0, 0.0) * bodies[index].distance * 50.0,
                bodies[index].angle + bodies[index].speed * total_time * 0.25,
            );
            self.compute_local_pos(bodies[index].parent, total_time) + relative
        }
    }
    fn compute_pos(&self, index: usize, total_time: f32) -> P2 {
        let value = self.compute_local_pos(index, total_time);

        self.world_to_screen(value)
    }
}

impl EventHandler<GameError> for Viewer {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn key_up_event(&mut self, ctx: &mut Context, input: KeyInput) -> Result<(), GameError> {
        match input.keycode {
            Some(ggez::input::keyboard::KeyCode::R) => {
                self.seed = 0;
                self.system = load_system(self.seed);
            }
            Some(ggez::input::keyboard::KeyCode::S) => {
                self.seed += 1;
                self.system = load_system(self.seed);
            }
            Some(ggez::input::keyboard::KeyCode::A) => {
                if self.seed > 0 {
                    self.seed -= 1;
                    self.system = load_system(self.seed);
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), GameError> {
        if button == MouseButton::Right {
            let v2screen = P2::new(x, y);
            self.target = self.screen_to_world(v2screen);
        }
        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), GameError> {
        if button == MouseButton::Left {
            let total_time = ggez::timer::time_since_start(ctx).as_secs_f32();
            let mouse_pos = P2::new(x, y);
            let mut min_index = None;
            let mut min_dist = None;

            for (i, b) in self.system.bodies.iter().enumerate() {
                let pos = self.compute_pos(i, total_time);
                let radius = BODY_SIZE * b.size * self.zoom;

                let dist = pos.distance(mouse_pos);
                if min_dist.is_none() || dist < min_dist.unwrap() {
                    min_dist = Some(dist);
                    min_index = Some(i);
                }
            }

            if let Some(i) = &min_index {
                let b = &self.system.bodies[*i];

                let text = {
                    match &b.desc {
                        BodyDesc::Planet {
                            atmosphere,
                            gravity,
                            biome,
                            ocean,
                            ..
                        } => format!(
                            "planet {}\nsize: {}\natmosphere: {}\ngravity: {}\nbiome: {}\nocean: {}",
                            i, b.size, atmosphere, gravity, biome, ocean
                        ),
                        BodyDesc::Star { .. } => "star".to_string(),
                        _ => "unknown".to_string(),
                    }
                };
                let selected = SelectedObject {
                    index: *i,
                    text: ggez::graphics::Text::new(text),
                };
                self.selected = Some(selected);
            }
        } else if button == MouseButton::Right {
            self.selected = None;
        }

        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        ctx: &mut Context,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
    ) -> Result<(), GameError> {
        if ctx.mouse.button_pressed(MouseButton::Left) {
            self.center.x += dx;
            self.center.y += dy;
        }
        Ok(())
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) -> Result<(), GameError> {
        self.zoom *= 1.0 + (y as f32) / 50.0;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
        let total_time = ctx.time.time_since_start().as_secs_f32();

        for (i, b) in self.system.bodies.iter().enumerate() {
            if i == 0 {
                let pos = self.compute_pos(i, total_time);

                let mesh = ggez::graphics::Mesh::new_circle(
                    ctx,
                    DrawMode::fill(),
                    pos,
                    BODY_SIZE * b.size * self.zoom,
                    1.0,
                    Color::WHITE,
                )?;
                canvas.draw(&mesh, graphics::DrawParam::default());
            } else {
                let pos = self.compute_pos(i, total_time);

                // draw body
                let mesh = ggez::graphics::Mesh::new_circle(
                    ctx,
                    DrawMode::fill(),
                    pos,
                    BODY_SIZE * b.size * self.zoom,
                    1.0,
                    Color::BLUE,
                )?;
                canvas.draw(&mesh, graphics::DrawParam::default());

                // draw orbit
                let parent_pos = self.compute_pos(b.parent, total_time);
                let dist = pos.distance(parent_pos);

                let mesh = ggez::graphics::Mesh::new_circle(
                    ctx,
                    DrawMode::stroke(1.0),
                    parent_pos,
                    dist,
                    1.0,
                    Color::RED,
                )?;
                canvas.draw(&mesh, graphics::DrawParam::default());

                // draw parent line
                let mesh =
                    ggez::graphics::Mesh::new_line(ctx, &[pos, parent_pos], 1.0, Color::GREEN)?;
                canvas.draw(&mesh, graphics::DrawParam::default());
            }
        }

        match &self.selected {
            Some(selected) => {
                let color_uibg = ggez::graphics::Color::from_rgb(239, 71, 111);
                let color_uitext = ggez::graphics::Color::from_rgb(255, 209, 102);
                let rect = Rect::new(
                    0.0,
                    3.0 * self.screen_height as f32 / 4.0,
                    self.screen_width as f32,
                    self.screen_height as f32 / 4.0,
                );
                let mesh =
                    ggez::graphics::Mesh::new_rectangle(ctx, DrawMode::fill(), rect, color_uibg)?;
                canvas.draw(&mesh, graphics::DrawParam::default());
                canvas.draw(
                    &selected.text,
                    DrawParam::default()
                        .dest(P2::new(rect.x + 20.0, rect.y + 20.0))
                        .color(color_uitext),
                );
            }
            _ => {}
        }

        // draw target
        let mesh = ggez::graphics::Mesh::new_circle(
            ctx,
            DrawMode::stroke(1.0),
            self.target * self.zoom + self.center,
            1.0,
            1.0,
            Color::YELLOW,
        )?;
        canvas.draw(&mesh, graphics::DrawParam::default());
        canvas.finish(ctx)
    }
}
