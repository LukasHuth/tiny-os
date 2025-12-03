use core::ops::{Deref, DerefMut};

use crate::{tetris::screen::Screen, utility::{self, time}, vga_buffer::Color};

mod screen;

pub struct Tetris {
    screen: Screen,
    last_update: u64,
    state: State,
}
enum State {
    Ingame(GameState),
    Menu(MenuState),
}
impl Deref for State {
    type Target = dyn StateBehaviour;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Ingame(state) => state,
            Self::Menu(state) => state,
        }
    }
}
impl DerefMut for State {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Ingame(state) => state,
            Self::Menu(state) => state,
        }
    }
}
trait StateBehaviour {
    fn update(&mut self) -> Option<State>;
    fn draw(&self, screen: &mut Screen);
    fn init(&mut self, screen: &mut Screen);
}
mod menu;
use menu::MenuState;
mod game;
use game::GameState;
const TIMESTEP: u64 = 1000 / 60;
impl Tetris {
    pub fn new() -> Self {
        let mut screen = Screen::new();
        screen.draw();
        utility::rand::init();
        Self {
            screen,
            last_update: time().saturating_sub(TIMESTEP),
            state: State::Menu(MenuState::new()),
        }
    }
    pub fn run(mut self) {
        loop {
            self.update();
            self.draw();
            x86_64::instructions::hlt();
        }
    }
    fn draw(&mut self) {
        self.state.draw(&mut self.screen);
    }
    fn update(&mut self) {
        let mut delta = time() - self.last_update;
        while delta > TIMESTEP {
            self.last_update += TIMESTEP;
            delta -= TIMESTEP;
            if let Some(next_state) = self.state.update() {
                self.state = next_state;
                self.state.init(&mut self.screen);
            }
        }
    }
}
