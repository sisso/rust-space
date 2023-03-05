use commons::math::{Transform2, V2};
use commons::{math, unwrap_or_continue, unwrap_or_return};
use ggegui::{egui, Gui};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, EventHandler, MouseButton};
use ggez::graphics::{self, Canvas, Color, DrawMode, DrawParam, Drawable, StrokeOptions};
use ggez::{Context, ContextBuilder, GameError, GameResult};
use space_flap;
use space_flap::{EventKind, ObjData, SectorData};

const COLOR_FLEET: Color = Color::RED;
const COLOR_ASTEROID: Color = Color::MAGENTA;
const COLOR_STATION: Color = Color::GREEN;
const COLOR_JUMP: Color = Color::BLUE;
const COLOR_ASTRO: Color = Color::YELLOW;
const COLOR_UNKNOWN: Color = Color::WHITE;

fn main() {
    env_logger::builder()
        .filter(None, log::LevelFilter::Warn)
        .filter(Some("world_view"), log::LevelFilter::Info)
        .filter(Some("space_flap"), log::LevelFilter::Info)
        .filter(Some("space_domain"), log::LevelFilter::Info)
        .filter(Some("space_domain::game::loader"), log::LevelFilter::Trace)
        .init();

    let args = vec!["--size", "2", "--fleets", "2"]
        .into_iter()
        .map(String::from)
        .collect();
    let sg = space_flap::SpaceGame::new(args);

    // Make a Context.
    let (ctx, event_loop) = ContextBuilder::new("my_game", "Cool Game Author")
        .window_mode(WindowMode::default().dimensions(1940.0, 1080.0))
        .build()
        .expect("aieee, could not create ggez context!");

    let sector_view_transform = get_sector_transform(&ctx);

    let my_game = State {
        egui_backend: Gui::default(),
        game: sg,
        screen: StateScreen::Galaxy,
        selected_sector: 0,
        selected_fleet: 0,
        sector_view_transform: sector_view_transform,
        time_speed: TimeSpeed::Normal,
        ui: Default::default(),
    };

    event::run(ctx, event_loop, my_game);
}

enum StateScreen {
    Sector(space_flap::Id),
    Galaxy,
    Fleet(space_flap::Id),
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum TimeSpeed {
    Pause,
    Normal,
}

#[derive(Default)]
struct Ui {
    dragging_start: Option<mint::Point2<f32>>,
    mouse_wheel: Option<f32>,
}

struct State {
    game: space_flap::SpaceGame,
    screen: StateScreen,
    selected_sector: usize,
    selected_fleet: usize,
    egui_backend: Gui,
    sector_view_transform: Transform2,
    time_speed: TimeSpeed,
    ui: Ui,
}

impl State {
    fn draw_fleet_sector(
        &mut self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        screen_size: (f32, f32),
        fleet_id: space_flap::Id,
    ) -> GameResult<()> {
        let Some(fleet) = self.game.get_obj(fleet_id)
        else {
            self.screen = StateScreen::Galaxy;
            return Ok(())
        };

        self.draw_sector(ctx, canvas, screen_size, fleet.get_sector_id())
    }

    fn draw_sector(
        &self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        screen_size: (f32, f32),
        sector_id: space_flap::Id,
    ) -> GameResult<()> {
        let at_sector = self.game.list_at_sector(sector_id);

        // draw orbits
        for obj_id in &at_sector {
            let obj = unwrap_or_continue!(self.game.get_obj(*obj_id));
            if !obj.is_astro() {
                continue;
            }

            let orbit = unwrap_or_continue!(obj.get_orbit());
            let radius = orbit.get_radius();
            if radius < 0.00001 {
                continue;
            }

            let parent_coords = orbit.get_parent_pos();
            let (parent_x, parent_y) = point_to_screen(&self.sector_view_transform, parent_coords);

            let sw_radius = length_to_screen(&self.sector_view_transform, radius);

            let orbit_circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::Stroke(graphics::StrokeOptions::default()),
                [0.0, 0.0],
                sw_radius,
                1.0,
                Color::WHITE,
            )?;

            canvas.draw(
                &orbit_circle,
                DrawParam::new()
                    .dest([parent_x, parent_y])
                    .color(Color::WHITE),
            );
        }

        // draw objects
        let planet_circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            [0.0, 0.0],
            3.0,
            1.0,
            Color::WHITE,
        )?;

        for obj_id in at_sector {
            let obj = unwrap_or_continue!(self.game.get_obj(obj_id));

            let color = Self::resolve_color(&obj);

            let coords = obj.get_coords();
            let (wx, wy) = point_to_screen(&self.sector_view_transform, coords);
            canvas.draw(&planet_circle, DrawParam::new().dest([wx, wy]).color(color));
        }

        // draw legends
        {
            let border = 10.0;
            let padding = 2.0;
            let x = border;
            let mut y = screen_size.1 - border;

            let mut list = vec![
                (COLOR_FLEET, "Fleet"),
                (COLOR_ASTRO, "Astronomic Body"),
                (COLOR_ASTEROID, "Asteroid"),
                (COLOR_STATION, "Station"),
                (COLOR_JUMP, "Jump Point"),
                (COLOR_UNKNOWN, "Unknown"),
            ];
            list.reverse();

            for (color, label) in list {
                let text = graphics::Text::new(label);
                y -= text.measure(&ctx.gfx)?.y;
                canvas.draw(&text, DrawParam::new().dest([x, y]).color(color));
                y -= padding;
            }
        }

        Ok(())
    }

    fn resolve_color(obj: &ObjData) -> Color {
        match (
            obj.is_fleet(),
            obj.is_asteroid(),
            obj.is_station(),
            obj.is_jump(),
            obj.is_astro(),
        ) {
            (true, _, _, _, _) => COLOR_FLEET,
            (_, true, _, _, _) => COLOR_ASTEROID,
            (_, _, true, _, _) => COLOR_STATION,
            (_, _, _, true, _) => COLOR_JUMP,
            (_, _, _, _, true) => COLOR_ASTRO,
            _ => COLOR_UNKNOWN,
        }
    }

    fn draw_galaxy(
        &mut self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        screen_size: (f32, f32),
    ) -> GameResult<()> {
        let sectors = self.game.get_sectors();

        let dimension = f64::sqrt(sectors.len() as f64) as usize;

        let border = 50.0;
        let space = 100.0;
        let boxsize =
            (screen_size.0 - border * 2.0 - space * (dimension as f32 - 1.0)) / dimension as f32;

        // let (cx, cy) = (border, border);

        let mesh = graphics::Mesh::new_rounded_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect {
                x: 0.0,
                y: 0.0,
                w: boxsize,
                h: boxsize,
            },
            2.0,
            Color::WHITE,
        )?;

        for i in 0..dimension {
            for j in 0..dimension {
                let cx = border + (i as f32) * (space + boxsize);
                let cy = border + (j as f32) * (space + boxsize);
                canvas.draw(&mesh, DrawParam::new().dest([cx, cy]));
            }
        }

        Ok(())
    }

    /// return true if mouse is currently hovering a gui element
    fn draw_gui(&mut self, ctx: &mut Context) -> bool {
        let sectors = self.game.get_sectors();
        let fleets = self.game.get_fleets();

        let egui_ctx = self.egui_backend.ctx();
        let result = egui::Window::new("egui-window").show(&egui_ctx, |ui| {
            match self.time_speed {
                TimeSpeed::Pause => {
                    if ui.button("time on").clicked() {
                        self.time_speed = TimeSpeed::Normal;
                    }
                }
                TimeSpeed::Normal => {
                    if ui.button("pause").clicked() {
                        self.time_speed = TimeSpeed::Pause;
                    }
                }
            }

            let sector_resp = egui::ComboBox::from_label("Sector").show_index(
                ui,
                &mut self.selected_sector,
                sectors.len(),
                |i| format!("{}{}", i, sector_text(&sectors[i])),
            );

            if sector_resp.changed() {
                self.screen = StateScreen::Sector(sectors[self.selected_sector].get_id());
            }

            if ui.button("back").clicked() {
                self.screen = StateScreen::Galaxy;
            }

            egui::ComboBox::from_label("Fleets").show_index(
                ui,
                &mut self.selected_fleet,
                fleets.len(),
                |i| format!("{}{}", i, fleet_text(&sectors, &fleets[i])),
            );

            if ui.button("select").clicked() {
                self.screen = StateScreen::Fleet(fleets[self.selected_fleet].get_id());
            }
        });

        self.egui_backend.update(ctx);

        egui_ctx.is_using_pointer() || egui_ctx.is_pointer_over_area()
    }

    fn handle_inputs(&mut self, ctx: &mut Context, hovering_gui: bool) -> GameResult<()> {
        if hovering_gui {
            return Ok(());
        }

        let mouse_pos = ctx.mouse.position();

        if ctx.mouse.button_pressed(MouseButton::Right) {
            match self.ui.dragging_start {
                Some(start) => {
                    let delta = ctx.mouse.delta();
                    log::info!("dragging {delta:?}");
                    self.sector_view_transform
                        .translate(math::V2::new(delta.x, delta.y));
                }
                None => {
                    self.ui.dragging_start = Some(mouse_pos);
                }
            }
        } else {
            self.ui.dragging_start = None;
        }

        if let Some(wheel) = self.ui.mouse_wheel.take() {
            let sensitivity = 0.5;
            self.sector_view_transform.scale(1.0 + wheel * sensitivity);
        }

        let button_released = ctx.mouse.button_just_released(MouseButton::Left);
        if button_released {
            log::info!("{:?}", mouse_pos);
        }

        Ok(())
    }
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let delta_time = ctx.time.delta().as_secs_f32();

        // update game tick
        if self.time_speed == TimeSpeed::Normal {
            self.game.update(delta_time);
        }

        // collect events
        for event in self.game.take_events() {
            match event.get_kind() {
                EventKind::Add => {}
                EventKind::Move => {}
                EventKind::Jump => {
                    // let d = unwrap_or_continue!(self.sg.get_obj(event.get_id()));
                    // let sid = d.get_sector_id();
                    // let sectors = self.sg.get_sectors();
                    // let index = sectors.iter().position(|i| i.get_id() == sid);
                    // println!(
                    //     "{} move at sector {}",
                    //     d.get_id(),
                    //     index.unwrap_or_default()
                    // );
                }
                EventKind::Dock => {}
                EventKind::Undock => {}
            }
        }

        let hovering = self.draw_gui(ctx);
        self.handle_inputs(ctx, hovering);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let tick = ctx.time.ticks();
        let delta_time = ctx.time.average_delta().as_secs_f32();

        let mut canvas = Canvas::from_frame(ctx, graphics::Color::BLACK);
        let screen_size = ctx.gfx.window().inner_size();
        let screen_size = (screen_size.width as f32, screen_size.height as f32);

        let text = graphics::Text::new(format!("{} {}", tick, delta_time));
        canvas.draw(&text, DrawParam::default().color(Color::WHITE));

        match self.screen {
            StateScreen::Sector(sector_id) => {
                self.draw_sector(ctx, &mut canvas, screen_size, sector_id)?;
            }
            StateScreen::Galaxy => {
                self.draw_galaxy(ctx, &mut canvas, screen_size)?;
            }
            StateScreen::Fleet(fleet_id) => {
                self.draw_fleet_sector(ctx, &mut canvas, screen_size, fleet_id)?;
            }
        }

        canvas.draw(&self.egui_backend, DrawParam::new());

        canvas.finish(ctx)
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: f32, y: f32) -> Result<(), GameError> {
        self.ui.mouse_wheel = Some(y);
        Ok(())
    }
}

fn get_sector_transform(ctx: &Context) -> Transform2 {
    let size = ctx.gfx.window().inner_size();
    let (w, h) = (size.width as f32, size.height as f32);
    let trans = math::P2::new(w * 0.5, h * 0.5);
    let scale = w / 20.0; // lets fit -10.0 to 10.0 grid
    Transform2::new(trans, scale, 0.0)
}

fn length_to_screen(transform: &Transform2, length: f32) -> f32 {
    let v2 = transform.get_similarity() * V2::new(length, 0.0);
    v2.magnitude()
}

fn point_to_screen(transform: &Transform2, pos: (f32, f32)) -> (f32, f32) {
    let p = transform.get_similarity() * math::P2::new(pos.0, pos.1);
    (p.x, p.y)
}

fn sector_text(sd: &space_flap::SectorData) -> String {
    format!("({},{})", sd.get_coords().0, sd.get_coords().1)
}

fn fleet_text(sectors: &Vec<SectorData>, d: &space_flap::ObjData) -> String {
    let sector_index = sectors
        .iter()
        .position(|s| s.get_id() == d.get_sector_id())
        .map(|id| format!("{}", id))
        .unwrap_or("None".to_string());

    format!(
        "{} {}/{:.1},{:.1}",
        d.get_id(),
        sector_index,
        d.get_coords().0,
        d.get_coords().1
    )
}
