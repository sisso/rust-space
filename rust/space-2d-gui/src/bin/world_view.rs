use approx::RelativeEq;
use commons::math::V2I;
use commons::{unwrap_or_continue, unwrap_or_return};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawMode, DrawParam, StrokeOptions, Transform};
use ggez::{Context, ContextBuilder, GameError, GameResult};
use ggez_egui::{egui, EguiBackend};
use space_flap;
use space_flap::{EventKind, ObjData, ObjKind, SectorData};

fn main() {
    env_logger::builder()
        .filter(None, log::LevelFilter::Warn)
        .filter(Some("world_view"), log::LevelFilter::Info)
        .filter(Some("space_flap"), log::LevelFilter::Info)
        .filter(Some("space_domain"), log::LevelFilter::Warn)
        .init();

    let args = vec!["--size", "4", "--fleets", "2"]
        .into_iter()
        .map(String::from)
        .collect();
    let sg = space_flap::SpaceGame::new(args);

    // Make a Context.
    let (ctx, event_loop) = ContextBuilder::new("my_game", "Cool Game Author")
        .build()
        .expect("aieee, could not create ggez context!");

    let my_game = State {
        egui_backend: EguiBackend::default(),
        sg: sg,
        screen: StateScreen::Galaxy,
        selected_sector: 0,
        selected_fleet: 0,
    };

    event::run(ctx, event_loop, my_game);
}

enum StateScreen {
    Sector(space_flap::Id),
    Galaxy,
}

struct State {
    sg: space_flap::SpaceGame,
    screen: StateScreen,
    selected_sector: usize,
    selected_fleet: usize,
    egui_backend: EguiBackend,
}

fn length_to_screen(screen_size: (f32, f32), length: f32) -> f32 {
    commons::math::map_value(length, 0.0, 20.0, 0.0, screen_size.0)
}

fn point_to_screen(screen_size: (f32, f32), pos: (f32, f32)) -> (f32, f32) {
    (
        commons::math::map_value(pos.0, -10.0, 10.0, 0.0, screen_size.0),
        commons::math::map_value(pos.1, -10.0, 10.0, 0.0, screen_size.1),
    )
}

impl State {
    fn draw_sector(
        &self,
        ctx: &mut Context,
        screen_size: (f32, f32),
        sector_id: space_flap::Id,
    ) -> GameResult<()> {
        let at_sector = self.sg.list_at_sector(sector_id);

        // draw orbits
        log::info!("start drawing orbits");
        for obj_id in &at_sector {
            let obj = unwrap_or_continue!(self.sg.get_obj(*obj_id));
            if !obj.is_astro() {
                continue;
            }

            let orbit = obj.get_orbit().unwrap();
            let radius = orbit.get_radius();
            if radius < 0.00001 {
                continue;
            }

            let parent_coords = orbit.get_parent_pos();
            let (parent_x, parent_y) = point_to_screen(screen_size, parent_coords);

            let sw_radius = length_to_screen(screen_size, radius);

            let orbit_circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::Stroke(StrokeOptions::default()),
                [0.0, 0.0],
                sw_radius,
                1.0,
                Color::WHITE,
            )?;

            graphics::draw(ctx, &orbit_circle, ([parent_x, parent_y], Color::WHITE))?;
        }

        // draw objects
        let planet_circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            [0.0, 0.0],
            4.0,
            1.0,
            Color::WHITE,
        )?;

        for obj_id in at_sector {
            let obj = unwrap_or_continue!(self.sg.get_obj(obj_id));

            let color = match (
                obj.is_fleet(),
                obj.is_asteroid(),
                obj.is_station(),
                obj.is_jump(),
                obj.is_astro(),
            ) {
                (true, _, _, _, _) => Color::RED,
                (_, true, _, _, _) => Color::MAGENTA,
                (_, _, true, _, _) => Color::GREEN,
                (_, _, _, true, _) => Color::BLUE,
                (_, _, _, _, true) => Color::YELLOW,
                _ => Color::WHITE,
            };

            let coords = obj.get_coords();
            let (wx, wy) = point_to_screen(screen_size, coords);
            graphics::draw(ctx, &planet_circle, ([wx, wy], color))?;
        }

        Ok(())
    }

    fn draw_galaxy(
        &mut self,
        ctx: &mut Context,
        screen_size: (f32, f32),
        sectors: Vec<SectorData>,
    ) -> GameResult<()> {
        let dimension = f64::sqrt(sectors.len() as f64) as usize;

        let border = 50.0;
        let space = 100.0;
        let boxsize =
            (screen_size.0 - border * 2.0 - space * (dimension as f32 - 1.0)) / dimension as f32;

        // let (cx, cy) = (border, border);

        let mesh = graphics::Mesh::new_rounded_rectangle(
            ctx,
            DrawMode::fill(),
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
                graphics::draw(ctx, &mesh, ([cx, cy],))?;
            }
        }

        Ok(())
    }
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

impl EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::BLACK);

        let screen_size = ggez::graphics::window(ctx).inner_size();
        let screen_size = (screen_size.width as f32, screen_size.height as f32);

        let tick = ggez::timer::ticks(ctx);

        let delta_time = ggez::timer::delta(ctx).as_secs_f32();
        self.sg.update(delta_time);
        for event in self.sg.take_events() {
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

        let sectors = self.sg.get_sectors();
        let fleets = self.sg.get_fleets();

        let egui_ctx = self.egui_backend.ctx();
        egui::Window::new("egui-window").show(&egui_ctx, |ui| {
            let sector_resp = egui::ComboBox::from_label("Sector").show_index(
                ui,
                &mut self.selected_sector,
                sectors.len(),
                |i| format!("{}{}", i, sector_text(&sectors[i])),
            );

            if sector_resp.changed() {
                self.screen = StateScreen::Sector(sectors[self.selected_sector].get_id());
            }

            egui::ComboBox::from_label("Fleets").show_index(
                ui,
                &mut self.selected_fleet,
                fleets.len(),
                |i| format!("{}{}", i, fleet_text(&sectors, &fleets[i])),
            );
        });

        let text = graphics::Text::new(format!("{} {}", tick, delta_time));
        graphics::draw(ctx, &text, DrawParam::default().color(Color::WHITE))?;

        match self.screen {
            StateScreen::Sector(sector_id) => {
                self.draw_sector(ctx, screen_size, sector_id)?;
            }
            StateScreen::Galaxy => {
                self.draw_galaxy(ctx, screen_size, sectors);
            }
        }

        graphics::draw(ctx, &self.egui_backend, ([0.0, 0.0],))?;

        graphics::present(ctx)
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: ggez::event::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.egui_backend.input.mouse_button_down_event(button);
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: ggez::event::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.egui_backend.input.mouse_button_up_event(button);
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.egui_backend.input.mouse_motion_event(x, y);
    }
}
