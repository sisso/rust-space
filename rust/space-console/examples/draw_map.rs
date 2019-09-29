#[allow(dead_code)]
extern crate tui;
extern crate termion;
extern crate space_domain;
#[macro_use]
extern crate space_macros;

mod util;

use std::io;
use std::time::Duration;

use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::Color;
use tui::widgets::canvas::{Canvas, Map, MapResolution, Rectangle, Points};
use tui::widgets::{Block, Borders, Widget};
use tui::Terminal;

use space_domain::utils::V2;
use crate::util::event::{Config, Event, Events};

enum ObjKind {
    JUMP,
    SHIP,
    STATION,
    ASTEROID,
}

struct Obj {
    kind: ObjKind,
    pos: V2,
}

struct Sector {
    label: String,
    objects: Vec<Obj>,
}

struct State {
    sectors: Vec<Sector>
}

fn main() {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode().unwrap();
//    let stdout = MouseTerminal::from(stdout);
//    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.hide_cursor().unwrap();

    // Setup event handlers
    let config = Config {
        tick_rate: Duration::from_millis(100),
        ..Default::default()
    };
    let events = Events::with_config(config);

    // App
    let mut state = State {
        sectors: vec![
            Sector {
                label: "Sector 0".to_string(),
                objects: vec![]
            },
            Sector {
                label: "Sector 0".to_string(),
                objects: vec![]
            },
        ]
    };


    debugf!("start");

    loop {
        terminal.draw(|mut f| {
            let window_title = "Sector";

            let percentage_per_sector = (100.0 / state.sectors.len() as f32) as u16;

            let mut layout_constraint = vec![];
            layout_constraint.resize(state.sectors.len(), Constraint::Percentage(percentage_per_sector));

            debugf!("constraints {:?}", layout_constraint);

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(layout_constraint)
                .split(f.size());

            Canvas::default()
                .block(Block::default().borders(Borders::ALL).title(window_title))
                .paint(|ctx| {
                    let points: Points = Points {
                        coords: &[(0.0, 0.0)],
                        color: Color::Red
                    };
                    ctx.draw(&points);
//                    ctx.print(app.x, -app.y, "You are here", Color::Yellow);
                })
                .x_bounds([-180.0, 180.0])
                .y_bounds([-90.0, 90.0])
                .render(&mut f, chunks[0]);

            Canvas::default()
                .block(Block::default().borders(Borders::ALL).title(window_title))
                .paint(|ctx| {
                    let points: Points = Points {
                        coords: &[(0.0, 0.0)],
                        color: Color::Red
                    };
                    ctx.draw(&points);
//                    ctx.print(app.x, -app.y, "You are here", Color::Yellow);
                })
                .x_bounds([-180.0, 180.0])
                .y_bounds([-90.0, 90.0])
                .render(&mut f, chunks[1]);
        }).unwrap();

        match events.next().unwrap() {
            Event::Input(input) => match input {
                Key::Char('q') | Key::Ctrl('c') => {
                    break;
                },
                other => {
                    debugf!("receive key {:?}", other);
                }
            },
            Event::Tick => {
            }
        }
    }

    debugf!("complete");
}
