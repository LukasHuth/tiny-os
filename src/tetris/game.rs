use core::ops::{Add, AddAssign, Sub, SubAssign};

use crate::{
    tetris::{screen::Screen, State, StateBehaviour},
    vga_buffer::Color,
};

pub struct GameState {
    color: Color,
    grid: [[Option<TetraminoBlockType>; 10]; 20],
    current_color_scheme: ColorScheme,
    current_tetramino: Tetramino,
    subticks_since_last_drop: u64,
    next_tetramino: Tetramino,
}
#[derive(Debug, Clone, Copy)]
struct Tetramino {
    rotation: Rotation,
    block_type: TetraminoBlockType,
    position: Position,
}
const TetraminoOffsets: [[[(usize, usize); 4]; 4]; 7] = [
    [
        // O
        [(0, 0), (0, 1), (1, 0), (1, 1)],
        [(0, 0), (0, 1), (1, 0), (1, 1)],
        [(0, 0), (0, 1), (1, 0), (1, 1)],
        [(0, 0), (0, 1), (1, 0), (1, 1)],
    ],
    [
        // I
        [(1, 0), (1, 1), (1, 2), (1, 3)],
        [(0, 1), (1, 1), (2, 1), (3, 1)],
        [(2, 0), (2, 1), (2, 2), (2, 3)],
        [(0, 2), (1, 2), (2, 2), (3, 2)],
    ],
    [
        // J
        [(1, 0), (1, 1), (1, 2), (0, 2)],
        [(0, 1), (1, 1), (2, 1), (0, 0)], // TODO
        [(1, 0), (1, 1), (1, 2), (2, 0)],
        [(0, 1), (1, 1), (2, 1), (2, 2)], // TODO
    ],
    [
        // L TODO: adjust (currently J Copied)
        [(1, 0), (1, 1), (1, 2), (0, 2)],
        [(0, 1), (1, 1), (2, 1), (0, 0)], // TODO
        [(1, 0), (1, 1), (1, 2), (2, 0)],
        [(0, 1), (1, 1), (2, 1), (2, 2)], // TODO
    ],
    [
        // S TODO: adjust (currently J Copied)
        [(1, 0), (1, 1), (1, 2), (0, 2)],
        [(0, 1), (1, 1), (2, 1), (0, 0)], // TODO
        [(1, 0), (1, 1), (1, 2), (2, 0)],
        [(0, 1), (1, 1), (2, 1), (2, 2)], // TODO
    ],
    [
        // Z TODO: adjust (currently J Copied)
        [(1, 0), (1, 1), (1, 2), (0, 2)],
        [(0, 1), (1, 1), (2, 1), (0, 0)], // TODO
        [(1, 0), (1, 1), (1, 2), (2, 0)],
        [(0, 1), (1, 1), (2, 1), (2, 2)], // TODO
    ],
    [
        // T TODO: adjust (currently J Copied)
        [(1, 0), (1, 1), (1, 2), (0, 2)],
        [(0, 1), (1, 1), (2, 1), (0, 0)], // TODO
        [(1, 0), (1, 1), (1, 2), (2, 0)],
        [(0, 1), (1, 1), (2, 1), (2, 2)], // TODO
    ],
];
impl Tetramino {
    fn random() -> Self {
        let block_type = {
            let random = (crate::utility::rand::rand() & 0b111) as u8;
            match random {
                0 => TetraminoBlockType::T,
                1 => TetraminoBlockType::S,
                2 => TetraminoBlockType::J,
                3 => TetraminoBlockType::I,
                4 => TetraminoBlockType::O,
                5 => TetraminoBlockType::L,
                6 => TetraminoBlockType::Z,
                7 => return Self::random(),
                _ => unreachable!(),
            }
        };
        Self {
            rotation: Rotation::Up,
            block_type,
            position: Position { x: 4, y: 0 },
        }
    }
    fn get_used_blocks(&self) -> [Position; 4] {
        match self.block_type {
            TetraminoBlockType::O => [
                self.position,
                self.position + (0, 1),
                self.position + (1, 0),
                self.position + (1, 1),
            ],
            TetraminoBlockType::I => match self.rotation {
                Rotation::Up => [
                    self.position + (1, 0),
                    self.position + (1, 1),
                    self.position + (1, 2),
                    self.position + (1, 3),
                ],
                _ => todo!(),
            },
            TetraminoBlockType::J => match self.rotation {
                Rotation::Up => [
                    self.position + (1, 0),
                    self.position + (1, 1),
                    self.position + (1, 2),
                    self.position + (0, 2),
                ],
                _ => todo!(),
            },
            TetraminoBlockType::L => match self.rotation {
                Rotation::Up => [
                    self.position + (1, 0),
                    self.position + (1, 1),
                    self.position + (1, 2),
                    self.position + (2, 2),
                ],
                _ => todo!(),
            },
            TetraminoBlockType::S => match self.rotation {
                Rotation::Up => [
                    self.position + (1, 0),
                    self.position + (2, 0),
                    self.position + (0, 1),
                    self.position + (1, 1),
                ],
                _ => todo!(),
            },
            TetraminoBlockType::Z => match self.rotation {
                Rotation::Up => [
                    self.position + (0, 0),
                    self.position + (1, 0),
                    self.position + (1, 1),
                    self.position + (2, 1),
                ],
                _ => todo!(),
            },
            TetraminoBlockType::T => match self.rotation {
                Rotation::Up => [
                    self.position + (0, 0),
                    self.position + (1, 0),
                    self.position + (2, 0),
                    self.position + (1, 1),
                ],
                _ => todo!(),
            },
        }
    }
    fn is_possible(&self, board: &[[Option<TetraminoBlockType>; 10]; 20]) -> bool {
        for pos in self.get_used_blocks() {
            if pos.y >= 20 || board[pos.y][pos.x].is_some() {
                return false;
            }
        }
        true
    }
}
#[derive(Debug, Clone, Copy)]
struct Position {
    x: usize,
    y: usize,
}
impl Add<(usize, usize)> for Position {
    type Output = Position;

    fn add(self, rhs: (usize, usize)) -> Self::Output {
        Position {
            x: self.x + rhs.0,
            y: self.y + rhs.1,
        }
    }
}
impl AddAssign<(usize, usize)> for Position {
    fn add_assign(&mut self, rhs: (usize, usize)) {
        self.x += rhs.0;
        self.y += rhs.1;
    }
}
impl Sub<(usize, usize)> for Position {
    type Output = Position;

    fn sub(self, rhs: (usize, usize)) -> Self::Output {
        Position {
            x: self.x - rhs.0,
            y: self.y - rhs.1,
        }
    }
}
impl SubAssign<(usize, usize)> for Position {
    fn sub_assign(&mut self, rhs: (usize, usize)) {
        self.x -= rhs.0;
        self.y -= rhs.1;
    }
}
#[derive(Debug, Clone, Copy)]
enum Rotation {
    Up,
    Right,
    Down,
    Left,
}
enum ColorScheme {
    First,
}
impl ColorScheme {
    pub const fn get_color(&self, tbt: &TetraminoBlockType) -> Color {
        match self {
            Self::First => match tbt {
                TetraminoBlockType::I => Color::LightCyan,
                TetraminoBlockType::O => Color::Yellow,
                TetraminoBlockType::T => Color::Pink,
                TetraminoBlockType::J => Color::LightBlue,
                TetraminoBlockType::L => Color::Brown,
                TetraminoBlockType::S => Color::LightGreen,
                TetraminoBlockType::Z => Color::LightRed,
            },
        }
    }
}
#[derive(Debug, Clone, Copy)]
enum TetraminoBlockType {
    I,
    O,
    T,
    J,
    L,
    S,
    Z,
}
impl GameState {
    pub fn new() -> Self {
        Self {
            color: Color::Black,
            grid: [[None; 10]; 20],
            current_color_scheme: ColorScheme::First,
            current_tetramino: Tetramino::random(),
            next_tetramino: Tetramino::random(),
            subticks_since_last_drop: 0,
        }
    }
}
const SUBTICKS_TILL_DROP: u64 = 6 * 5;
impl StateBehaviour for GameState {
    fn update(&mut self) -> Option<State> {
        self.subticks_since_last_drop += 1;
        if self.subticks_since_last_drop < SUBTICKS_TILL_DROP {
            return None;
        }
        self.subticks_since_last_drop = 0;
        let initial_tetramino = self.current_tetramino;
        // todo: movement
        self.current_tetramino.position += (0, 1);
        if !self.current_tetramino.is_possible(&self.grid) {
            self.current_tetramino = initial_tetramino;
            for piece in self.current_tetramino.get_used_blocks() {
                self.grid[piece.y][piece.x] = Some(self.current_tetramino.block_type);
            }
            self.new_piece();
        } else {
        }
        None
    }

    fn draw(&self, screen: &mut Screen) {
        self.clear_window(screen);
        self.draw_grid(screen);
        self.draw_current_tetramino(screen);
        screen.draw();
    }

    fn init(&mut self, screen: &mut Screen) {
        self.draw_gui(screen);
    }
}
const GAME_WINDOW_WIDTH: usize = 20;
const GAME_WINDOW_HEIGHT: usize = 40;
impl GameState {
    fn new_piece(&mut self) {
        self.current_tetramino = self.next_tetramino;
        self.next_tetramino = Tetramino::random();
    }

    fn draw_gui(&self, screen: &mut Screen) {
        screen.clear(Color::LightGray);
        self.clear_window(screen);
    }
    fn clear_window(&self, screen: &mut Screen) {
        let game_window_start_width = (screen.width() / 2) - (GAME_WINDOW_WIDTH / 2);
        let game_window_start_height = screen.height() - GAME_WINDOW_HEIGHT - 1;
        let game_window_end_width = game_window_start_width + GAME_WINDOW_WIDTH;
        let game_window_end_height = game_window_start_height + GAME_WINDOW_HEIGHT;
        screen.fill(
            game_window_start_width,
            game_window_start_height,
            game_window_end_width,
            game_window_end_height,
            Color::Black,
        );
    }
    fn draw_grid(&self, screen: &mut Screen) {
        let game_window_start_width = (screen.width() / 2) - (GAME_WINDOW_WIDTH / 2);
        let game_window_start_height = screen.height() - GAME_WINDOW_HEIGHT - 1;
        for (y, pieces) in self.grid.iter().enumerate() {
            for (x, piece) in pieces.iter().enumerate() {
                if let Some(piece) = piece {
                    let x_start = game_window_start_width + (x * 2);
                    let y_start = game_window_start_height + (y * 2);
                    let x_end = x_start + 1;
                    let y_end = y_start + 1;
                    screen.fill(
                        x_start,
                        y_start,
                        x_end,
                        y_end,
                        self.current_color_scheme.get_color(piece),
                    );
                }
            }
        }
    }
    fn draw_current_tetramino(&self, screen: &mut Screen) {
        let game_window_start_width = (screen.width() / 2) - (GAME_WINDOW_WIDTH / 2);
        let game_window_start_height = screen.height() - GAME_WINDOW_HEIGHT - 1;
        let current_tetramino_color = self
            .current_color_scheme
            .get_color(&self.current_tetramino.block_type);
        for piece in self.current_tetramino.get_used_blocks() {
            let x_start = game_window_start_width + (piece.x * 2);
            let y_start = game_window_start_height + (piece.y * 2);
            let x_end = x_start + 1;
            let y_end = y_start + 1;
            screen.fill(x_start, y_start, x_end, y_end, current_tetramino_color);
        }
    }
}
