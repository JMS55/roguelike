#![windows_subsystem = "windows"]

mod components;
mod entities;
mod game;

use crate::game::Game;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::render::Texture;
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, Instant};

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let video_context = sdl_context.video().unwrap();
    let window = video_context.window("roguelike", 480, 480).build().unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut textures = fs::read_dir("assets")
        .unwrap()
        .filter_map(|file| {
            let path = file.unwrap().path();
            if path.extension().unwrap() != "png" {
                return None;
            }
            let filename = path
                .file_stem()
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap();
            let texture = texture_creator.load_texture(path).unwrap();
            Some((filename, texture))
        })
        .collect::<HashMap<String, Texture>>();
    let ttf_context = sdl2::ttf::init().unwrap();
    let font = ttf_context.load_font("assets/04B_03__.ttf", 16).unwrap();

    let seed = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .collect::<String>();
    let mut game = Game::new(seed.clone(), seed);
    let mut time_accumulator = Duration::from_secs(0);
    let mut previous_time = Instant::now();
    'main_loop: loop {
        // Handle window events
        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                break 'main_loop;
            }
        }

        // Update
        let current_time = Instant::now();
        time_accumulator += current_time - previous_time;
        previous_time = current_time;
        while time_accumulator >= Duration::from_nanos(16700000) {
            game.run(&event_pump.keyboard_state());
            time_accumulator -= Duration::from_nanos(16700000);
        }

        // Render
        canvas.clear();
        game.render(&mut canvas, &mut textures, &texture_creator, &font);
        canvas.present();
    }
}
