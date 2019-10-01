use std::io;
use std::io::Stdout;
use std::time::Duration;

use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::Color;
use tui::Terminal;
use tui::widgets::{Block, Borders, Widget};
use tui::widgets::canvas::{Canvas, Map, MapResolution, Points, Rectangle};

use events::{Config, Event, Events};
use space_domain::utils::V2;

mod events;

type TTerminal = Terminal<TermionBackend<AlternateScreen<RawTerminal<Stdout>>>>;

#[derive(Debug, Copy, Clone)]
pub enum GuiObjKind {
    JUMP,
    SHIP,
    STATION,
    ASTEROID,
}

impl GuiObjKind {
    pub fn color(self) -> Color {
        match self {
            GuiObjKind::SHIP => Color::Blue,
            GuiObjKind::STATION => Color::Green,
            GuiObjKind::ASTEROID => Color::Gray,
            GuiObjKind::JUMP => Color::Red,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GuiObj {
    pub kind: GuiObjKind,
    pub pos: V2,
}

#[derive(Debug, Clone)]
pub struct GuiSector {
    pub label: String,
    pub objects: Vec<GuiObj>,
}

pub struct Gui {
    terminal: TTerminal,
    events: Events,
    exit: bool,
}

pub trait ShowSectorView {
    fn get_sectors_len(&self) -> usize;

    fn get_sector(&self, sector_index: usize) -> GuiSector;
}

impl Gui {
    pub fn new(time_rate: Duration) -> Result<Self, std::io::Error> {
        let stdout = std::io::stdout().into_raw_mode()?;
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal  = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        let events = Events::with_config(Config {
            tick_rate: time_rate,
            ..Default::default()
        });

        let gui = Gui {
            terminal,
            events,
            exit: false
        };

        Ok(gui)
    }

    pub fn show_sectors(&mut self, view: &dyn ShowSectorView) {
        self.terminal.draw(|mut f| {
            let percentage_per_sector = (100.0 / view.get_sectors_len() as f32) as u16;

            let mut layout_constraint = vec![];
            layout_constraint.resize(view.get_sectors_len(), Constraint::Percentage(percentage_per_sector));

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(layout_constraint)
                .split(f.size());

            for index in 0..view.get_sectors_len() {
                let sector = view.get_sector(index);

                Canvas::default()
                    .block(Block::default().borders(Borders::ALL).title(sector.label.as_str()))
                    .paint(|ctx| {
                        for obj in &sector.objects {
                            let points: Points = Points {
                                coords: &[(obj.pos.x as f64, obj.pos.y as f64)],
                                color: obj.kind.color(),
                            };
                            ctx.draw(&points);
                        }
                    })
                    .x_bounds([-10.0, 10.0])
                    .y_bounds([-10.0, 10.0])
                    .render(&mut f, chunks[index]);
            }
        }).unwrap();

        match self.events.next().unwrap() {
            Event::Input(input) => match input {
                Key::Char('q') | Key::Ctrl('c') => {
                    self.exit = true;
                },
                other => {
                }
            },
            Event::Tick => {
                // ignore
            }
        }
    }

    pub fn exit(&self) -> bool {
        self.exit
    }
}