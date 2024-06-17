use core::{convert::Infallible, ops::Range};

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};

pub mod hsv;

pub const WIDTH: usize = 8;
pub const HEIGHT: usize = 32 * 6;
pub const NUM_LEDS: usize = WIDTH * HEIGHT;

#[derive(Debug)]
pub struct Display<T> {
    data: [T; NUM_LEDS],
}

impl<T: Copy + Default> Display<T> {
    pub fn new() -> Self {
        Self {
            data: [T::default(); NUM_LEDS],
        }
    }

    pub fn data(&self, range: Range<usize>) -> DisplayIterator<T> {
        DisplayIterator::new(self, range)
    }

    pub fn get_index(x: usize, y: usize) -> usize {
        if y % 2 == 1 {
            x + WIDTH * y
        } else {
            (WIDTH - 1 - x) + WIDTH * y
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> T {
        let index = Self::get_index(x, y);
        self.data[index]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: T) {
        let index = Self::get_index(x, y);
        self.data[index] = color;
    }
}

impl DrawTarget for Display<Rgb888> {
    type Color = Rgb888;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            let x = (WIDTH as i32 - 1) - pixel.0.y;
            let mut y = pixel.0.x % HEIGHT as i32;
            if y < 0 {
                y += HEIGHT as i32;
            }
            if x < 0 || x >= WIDTH as i32 {
                continue;
            }
            self.set_pixel(x as usize, y as usize, pixel.1);
        }
        Ok(())
    }
}

impl OriginDimensions for Display<Rgb888> {
    fn size(&self) -> Size {
        Size::new(HEIGHT as u32, WIDTH as u32)
    }
}

pub struct DisplayIterator<'a, T> {
    display: &'a Display<T>,
    range: Range<usize>,
    index: usize,
}

impl<'a, T: Copy + Default> DisplayIterator<'a, T> {
    pub fn new(display: &'a Display<T>, range: Range<usize>) -> Self {
        let index = range.start;
        Self {
            display,
            range,
            index,
        }
    }
}

impl<'a, T: Copy + Default> Iterator for DisplayIterator<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.range.end {
            return None;
        }

        self.index += 1;

        Some(self.display.data[self.index - 1])
    }
}

#[cfg(test)]
mod tests {
    use smart_leds::RGB8;

    use super::*;
    

    #[test]
    fn test_display() {
        let mut display = Display::new();
        display.set_pixel(0, 0, RGB8 { r: 1, g: 2, b: 3 });
        assert_eq!(display.get_pixel(0, 0), RGB8 { r: 1, g: 2, b: 3 });
    }

    #[test]
    fn test_zig_zag() {
        let mut display = Display::new();
        display.set_pixel(WIDTH - 1, 0, RGB8 { r: 1, g: 2, b: 3 });
        display.set_pixel(0, 1, RGB8 { r: 4, g: 5, b: 6 });
        assert_eq!(display.data[WIDTH - 1], RGB8 { r: 1, g: 2, b: 3 });
        assert_eq!(display.data[WIDTH], RGB8 { r: 4, g: 5, b: 6 });
    }
}
