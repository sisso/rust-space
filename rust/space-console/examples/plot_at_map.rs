extern crate rand;

use space_console::gui::Gui;
use std::time::Duration;
use termion::event::Key;
use space_console::gui::events::Event;
use tui::layout::{Constraint, Layout, Direction};
use tui::widgets::canvas::{Canvas, Points};
use tui::widgets::{Borders, Block, Widget};
use tui::style::Color;
use rand::Rng;


fn main() {
    let mut gui = Gui::new(Duration::from_millis(250)).unwrap();

    let mut asteroids = vec![];
    let mut random = rand::thread_rng();
    for cluster in 0..20 {
        let central_x = random.gen_range(-100.0, 100.0);
        let central_y = random.gen_range(-100.0, 100.0);

        for i in 0..25 {
            // // block
            // let x = random.gen_range(-10.0, 10.0);
            // let y = random.gen_range(-10.0, 10.0);

            // circular
            let pi = 3.14159f64;
            let angle = random.gen_range(0.0, 2.0 * pi);
            let dist = random.gen_range(0.0, 10.0);

            let x = dist * angle.sin();
            let y = dist * angle.cos();

            asteroids.push((x + central_x, y + central_y));
        }

    }

    gui.terminal.clear().unwrap();

    loop {
        gui.terminal.draw(|mut f| {
            let mut layout_constraint = vec![];
            layout_constraint.resize(
                2,
                Constraint::Percentage(50),
            );

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(layout_constraint)
                .split(f.size());

            Canvas::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Screen 0"),
                )
                .paint(|ctx| {
                    let points: Points = Points {
                        coords: asteroids.as_slice(),
                        color: Color::Blue,
                    };

                    ctx.draw(&points);

                })
                .x_bounds([-100.0, 100.0])
                .y_bounds([-100.0, 100.0])
                .render(&mut f, chunks[0]);
        });

        match gui.events.next().unwrap() {
            Event::Input(input) => match input {
                Key::Char('q') | Key::Ctrl('c') => {
                    break;
                }
                other => {}
            },
            Event::Tick => {
                // ignore
            }
        }
    }
}