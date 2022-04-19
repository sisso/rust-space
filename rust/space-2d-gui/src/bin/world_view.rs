use commons::math::V2I;
use commons::{unwrap_or_continue, unwrap_or_return};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawParam};
use ggez::{Context, ContextBuilder, GameResult};
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

    let args = vec!["--size", "4", "--fleets", "10"]
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
        selected_sector: 0,
        selected_sector_id: None,
        selected_fleet: 0,
        selected_fleet_id: None,
    };

    event::run(ctx, event_loop, my_game);
}

struct State {
    sg: space_flap::SpaceGame,
    selected_sector: usize,
    selected_sector_id: Option<space_flap::Id>,
    selected_fleet: usize,
    selected_fleet_id: Option<space_flap::Id>,
    egui_backend: EguiBackend,
}

impl State {
    fn draw_sector(&self, ctx: &mut Context) -> GameResult<()> {
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            [0.0, 0.0],
            4.0,
            1.0,
            Color::WHITE,
        )?;

        let screen_size = ggez::graphics::window(ctx).inner_size();
        let (sw, sh) = (screen_size.width as f32, screen_size.height as f32);

        let id = unwrap_or_return!(self.selected_sector_id, Ok(()));
        for obj_id in self.sg.list_at_sector(id) {
            let obj = unwrap_or_continue!(self.sg.get_obj(obj_id));

            let color = match obj.get_kind() {
                ObjKind::Fleet => Color::RED,
                ObjKind::Asteroid => Color::MAGENTA,
                ObjKind::Station => Color::GREEN,
                ObjKind::Jump => Color::BLUE,
            };

            let (x, y) = obj.get_coords();
            let wx = commons::math::map_value(x, -10.0, 10.0, 0.0, sw);
            let wy = commons::math::map_value(y, -10.0, 10.0, 0.0, sh);
            ggez::graphics::draw(ctx, &circle, ([wx, wy], color))?;
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
    fn update(&mut self, ctx: &mut Context) -> GameResult {
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
            egui::ComboBox::from_label("Sector").show_index(
                ui,
                &mut self.selected_sector,
                sectors.len(),
                |i| format!("{}{}", i, sector_text(&sectors[i])),
            );

            self.selected_sector_id = Some(sectors[self.selected_sector].get_id());

            egui::ComboBox::from_label("Fleets").show_index(
                ui,
                &mut self.selected_fleet,
                fleets.len(),
                |i| format!("{}{}", i, fleet_text(&sectors, &fleets[i])),
            );

            self.selected_fleet_id = Some(fleets[self.selected_fleet].get_id());
        });
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::BLACK);

        let delta_time = ggez::timer::delta(ctx).as_secs_f32();
        let tick = ggez::timer::ticks(ctx);

        let text = graphics::Text::new(format!("{} {}", tick, delta_time));
        graphics::draw(ctx, &text, DrawParam::default().color(Color::WHITE))?;

        self.draw_sector(ctx)?;

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
