use crate::tetris::screen::Screen;

mod screen;

pub struct Tetris {
    screen: Screen,
    last_update: u64,
}
impl Tetris {
    pub fn new() -> Self {
        let mut screen = Screen::new();
        screen.draw();
        Self {
            screen,
            last_update: 0,
        }
    }
    pub fn run(self) {
        loop {
            x86_64::instructions::hlt();
        }
    }
}
