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

#[derive(Debug, Copy, Clone)]
enum ObjKind {
    JUMP,
    SHIP,
    STATION,
    ASTEROID,
}

#[derive(Debug, Clone)]
struct Obj {
    kind: ObjKind,
    pos: V2,
}

#[derive(Debug, Clone)]
struct Sector {
    label: String,
    objects: Vec<Obj>,
}

#[derive(Debug, Clone)]
struct State {
    sectors: Vec<Sector>
}

fn main() {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode().unwrap();
//    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
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
                objects: vec![
                    Obj {
                        kind: ObjKind::JUMP,
                        pos: V2::new(7.0, 5.0)
                    },
                    Obj {
                        kind: ObjKind::ASTEROID,
                        pos: V2::new(-3.0, -5.0)
                    },
                ]
            },
            Sector {
                label: "Sector 1".to_string(),
                objects: vec![
                    Obj {
                        kind: ObjKind::JUMP,
                        pos: V2::new(-4.0, 2.0)
                    },
                    Obj {
                        kind: ObjKind::STATION,
                        pos: V2::new(0.0, -5.0)
                    },
                    Obj {
                        kind: ObjKind::SHIP,
                        pos: V2::new(0.0, 0.0)
                    },
                ]
            },
        ]
    };

    debugf!("start");

    fn color_from_kind(kind: ObjKind) -> Color {
        match kind {
            ObjKind::SHIP => Color::Blue,
            ObjKind::STATION => Color::Green,
            ObjKind::ASTEROID => Color::Gray,
            ObjKind::JUMP => Color::Red,
        }
    }

    loop {
        terminal.draw(|mut f| {
            let percentage_per_sector = (100.0 / state.sectors.len() as f32) as u16;

            let mut layout_constraint = vec![];
            layout_constraint.resize(state.sectors.len(), Constraint::Percentage(percentage_per_sector));

            debugf!("constraints {:?}", layout_constraint);

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(layout_constraint)
                .split(f.size());

            for (index, sector) in state.sectors.iter().enumerate() {
                Canvas::default()
                    .block(Block::default().borders(Borders::ALL).title(sector.label.as_str()))
                    .paint(|ctx| {
                        for obj in &sector.objects {
                            let points: Points = Points {
                                coords: &[(obj.pos.x as f64, obj.pos.y as f64)],
                                color: color_from_kind(obj.kind)
                            };
                            ctx.draw(&points);
                        }
                    })
                    .x_bounds([-10.0, 10.0])
                    .y_bounds([-10.0, 10.0])
                    .render(&mut f, chunks[index]);
            }
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
                for s in &mut state.sectors {
                    for o in &mut s.objects {
                        match o.kind {
                            ObjKind::SHIP => {
                                o.pos = o.pos.add(&V2::new(0.1, 0.0));
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    debugf!("complete");
}
