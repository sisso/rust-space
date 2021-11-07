extern crate rand;

use rand::{Rng};
use space_console::gui::events::Event;
use space_console::gui::Gui;
use std::time::Duration;
use termion::event::Key;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::Color;
use tui::widgets::canvas::{Canvas, Points};
use tui::widgets::{Block, Borders, Widget};

fn main() {
    let mut gui = Gui::new(Duration::from_millis(250)).unwrap();

    let mut asteroids = vec![];

    fn generate(asteroids: &mut Vec<(f64, f64)>) {
        let pi = 3.14159f64;
        let mut random = rand::thread_rng();

        // for cluster in 0..20 {
        //     let central_x = random.gen_range(-100.0, 100.0);
        //     let central_y = random.gen_range(-100.0, 100.0);
        //
        //     for i in 0..25 {
        //         // // block
        //         // let x = random.gen_range(-10.0, 10.0);
        //         // let y = random.gen_range(-10.0, 10.0);
        //
        //         // circular
        //         let pi = 3.14159f64;
        //         let angle = random.gen_range(0.0, 2.0 * pi);
        //         let dist = random.gen_range(0.0, 10.0);
        //
        //         let x = dist * angle.sin();
        //         let y = dist * angle.cos();
        //
        //         asteroids.push((x + central_x, y + central_y));
        //     }
        // }

        let distance_slices = 5;
        let distance = 20.0;
        let distance_per_slice = distance / distance_slices as f64;

        let central_x = 0.0;
        let central_y = 0.0;

        for i_dist in 1..distance_slices {
            let angle_slices = i_dist * 4;
            let ang_per_slice = 2.0 * pi / angle_slices as f64;

            for i_angle in 0..angle_slices {
                let angle = i_angle as f64 * ang_per_slice;
                let dist = i_dist as f64 * distance_per_slice;

                // let rx = 0.0;
                // let ry = 0.0;
                let rx = random.gen_range(-distance_per_slice / 2.0, distance_per_slice / 2.0);
                let ry = random.gen_range(-distance_per_slice / 2.0, distance_per_slice / 2.0);

                let x = dist * angle.sin();
                let y = dist * angle.cos();

                asteroids.push((x + central_x + rx, y + central_y + ry));
                // asteroids.push((x + central_x, y + central_y))
            }
        }
    }

    generate(&mut asteroids);

    gui.terminal.clear().unwrap();

    loop {
        gui.terminal.draw(|mut f| {
            let mut layout_constraint = vec![];
            layout_constraint.resize(1, Constraint::Percentage(100));

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(layout_constraint)
                .split(f.size());

            Canvas::default()
                .block(Block::default().borders(Borders::ALL).title("Screen 0"))
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
                Key::Char('\n') => {
                    gui.terminal.clear().unwrap();
                    asteroids.clear();
                    generate(&mut asteroids);
                }
                _other => {}
            },
            Event::Tick => {
                // ignore
            }
        }
    }
}
