#![windows_subsystem = "windows"]

mod components;
mod entities;
mod game;
mod generate_dungeon;
mod spawn_enemies;
mod stages;

use sdl2::event::Event;
use stages::NewGameStage;
use stages::Stage;
use std::time::{Duration, Instant};

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let video_context = sdl_context.video().unwrap();
    let window = video_context.window("roguelike", 480, 480).build().unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut texture_creator = canvas.texture_creator();
    let ttf_context = sdl2::ttf::init().unwrap();
    let mut font = ttf_context.load_font("assets/04B_03__.ttf", 8).unwrap();

    let mut stage: Box<dyn Stage> = Box::new(NewGameStage {});
    let mut time_accumulator = Duration::from_secs(0);
    let mut previous_time = Instant::now();
    'main_loop: loop {
        {
            // Handle window events
            for event in event_pump.poll_iter() {
                if let Event::Quit { .. } = event {
                    break 'main_loop;
                }
            }

            // Input
            stage.input(&event_pump.keyboard_state());

            let current_time = Instant::now();
            time_accumulator += current_time - previous_time;
            previous_time = current_time;

            // Update
            while time_accumulator >= Duration::from_nanos(16700000) {
                stage = stage.update();
                time_accumulator -= Duration::from_nanos(16700000);
            }

            // Render
            canvas.clear();
            stage.render(&mut canvas, &mut texture_creator, &mut font);
            canvas.present();
        }
    }
}
