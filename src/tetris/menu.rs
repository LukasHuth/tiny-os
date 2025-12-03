use crate::tetris::{GameState, State, StateBehaviour, screen::Screen};

pub struct MenuState {
    selected_option: u8,
}
impl MenuState {
    pub fn new() -> Self {
        Self { selected_option: 0 }
    }
}
impl StateBehaviour for MenuState {
    fn update(&mut self) -> Option<State> {
        // TODO: Do menu management
        Some(State::Ingame(GameState::new()))
    }

    fn draw(&self, _screen: &mut Screen) {
        _screen.draw();
    }

    fn init(&mut self, _screen: &mut Screen) {
    }
}
