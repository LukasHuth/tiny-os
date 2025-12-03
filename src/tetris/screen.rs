use crate::vga_buffer::{self, Color, ColorCode};
pub struct Screen {
    pixels: [[Color; BUFFER_HEIGHT * 2]; BUFFER_WIDTH],
    printed_pixels: [[ColorCode; BUFFER_WIDTH]; BUFFER_HEIGHT],
}
impl Screen {
    pub fn new() -> Screen {
        Screen {
            pixels: [[Color::Black; vga_buffer::BUFFER_HEIGHT * 2]; vga_buffer::BUFFER_WIDTH],
            printed_pixels: [[ColorCode::new(Color::White, Color::White); BUFFER_WIDTH];
                BUFFER_HEIGHT],
        }
    }
    pub fn set_color(&mut self, x: usize, y: usize, color: Color) {
        self.pixels[x][y] = color;
    }
    pub fn fill(
        &mut self,
        x_start: usize,
        y_start: usize,
        x_end: usize,
        y_end: usize,
        color: Color,
    ) {
        assert!(
            x_start <= x_end,
            "x_start should be smaller or equal to x_end"
        );
        assert!(
            y_start <= y_end,
            "y_start should be smaller or equal to y_end"
        );
        for x in x_start..=x_end {
            for y in y_start..=y_end {
                self.set_color(x, y, color);
            }
        }
    }
    pub fn clear(&mut self, color: Color) {
        self.fill(0, 0, self.pixels.len() - 1, self.pixels[0].len() - 1, color);
    }
    pub fn draw(&mut self) {
        let pixel_map: [[ColorCode; BUFFER_WIDTH]; BUFFER_HEIGHT] = A.map(|arr| {
            arr.map(|(x, y)| ColorCode::new(self.pixels[x][y * 2 + 1], self.pixels[x][y * 2]))
        });
        let values_to_update = pixel_map
            .iter()
            .zip(self.printed_pixels.iter())
            .enumerate()
            .map(|(y, (arr0, arr1))| {
                arr0.iter()
                    .zip(arr1.iter())
                    .enumerate()
                    .filter_map(move |(x, (a, b))| if *a != *b { Some(((y, x), a)) } else { None })
            })
            .flatten();
        let writer = &mut vga_buffer::WRITER.lock();
        for ((y, x), colors) in values_to_update {
            writer.write_pixels(x as u8, y as u8, *colors);
        }
        self.printed_pixels = pixel_map;
    }

    pub(crate) fn width(&self) -> usize {
        self.pixels.len()
    }

    pub(crate) fn height(&self) -> usize {
        self.pixels[0].len()
    }
}
use vga_buffer::{BUFFER_HEIGHT, BUFFER_WIDTH};
const fn make_row(y: usize) -> [(usize, usize); BUFFER_WIDTH] {
    let mut row = [(0, 0); BUFFER_WIDTH];
    let mut x = 0;
    while x < BUFFER_WIDTH {
        row[x] = (x, y);
        x += 1;
    }
    row
}

const fn make_grid() -> [[(usize, usize); BUFFER_WIDTH]; BUFFER_HEIGHT] {
    let mut grid = [[(0, 0); BUFFER_WIDTH]; BUFFER_HEIGHT];
    let mut y = 0;
    while y < BUFFER_HEIGHT {
        grid[y] = make_row(y);
        y += 1;
    }
    grid
}

const A: [[(usize, usize); BUFFER_WIDTH]; BUFFER_HEIGHT] = make_grid();
