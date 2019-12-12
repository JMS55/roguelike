use crate::game::Game;
use sdl2::keyboard::KeyboardState;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::Font;
use sdl2::video::WindowContext;

pub trait Stage {
    fn input(&mut self, keyboard: &KeyboardState);

    fn update(&self) -> Option<Box<dyn Stage>>;

    fn render(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_creator: &mut TextureCreator<WindowContext>,
        font: &mut Font,
        delta_time: f64,
    );
}

pub struct NewGameStage {}

impl Stage for NewGameStage {
    fn input(&mut self, _: &KeyboardState) {}

    fn update(&self) -> Option<Box<dyn Stage>> {
        Some(Box::new(PlayerTurnStage { game: Game::new() }))
    }

    fn render(
        &mut self,
        canvas: &mut WindowCanvas,
        _: &mut TextureCreator<WindowContext>,
        _: &mut Font,
        _: f64,
    ) {
        canvas.clear();
    }
}

pub struct PlayerTurnStage {
    game: Game,
}

impl Stage for PlayerTurnStage {
    fn input(&mut self, keyboard: &KeyboardState) {}

    fn update(&self) -> Option<Box<dyn Stage>> {
        None
    }

    fn render(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_creator: &mut TextureCreator<WindowContext>,
        font: &mut Font,
        delta_time: f64,
    ) {
        self.game.render(canvas, texture_creator, font, delta_time);
    }
}
