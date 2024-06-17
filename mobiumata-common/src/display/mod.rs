use core::{convert::Infallible, ops::Range};

use embassy_futures::join::join;
use embassy_rp::peripherals::PIO1;
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use ws2812::Ws2812;

pub mod hsv;
pub mod ws2812;

pub const WIDTH: usize = 8;
pub const HEIGHT: usize = 32 * 6;
pub const NUM_LEDS: usize = WIDTH * HEIGHT;
const NUM_LEDS_PER_PIN: usize = NUM_LEDS / 2;

pub struct Display<'d> {
    data: [Rgb888; NUM_LEDS],
    ws2812_1: Ws2812<'d, PIO1, 0, NUM_LEDS_PER_PIN>,
    ws2812_2: Ws2812<'d, PIO1, 1, NUM_LEDS_PER_PIN>,
}

impl<'d> Display<'d> {
    pub fn new(
        ws2812_1: Ws2812<'d, PIO1, 0, NUM_LEDS_PER_PIN>,
        ws2812_2: Ws2812<'d, PIO1, 1, NUM_LEDS_PER_PIN>,
    ) -> Self {
        Self {
            data: [Rgb888::default(); NUM_LEDS],
            ws2812_1,
            ws2812_2,
        }
    }

    pub fn get_index(x: usize, y: usize) -> usize {
        if y % 2 == 1 {
            x + WIDTH * y
        } else {
            (WIDTH - 1 - x) + WIDTH * y
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Rgb888 {
        let index = Self::get_index(x, y);
        self.data[index]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Rgb888) {
        let index = Self::get_index(x, y);
        self.data[index] = color;
    }

    pub async fn flush(&mut self) {
        join(
            self.ws2812_1.write(self.data[0..NUM_LEDS_PER_PIN].iter().copied()),
            self.ws2812_2.write(self.data[NUM_LEDS_PER_PIN..NUM_LEDS].iter().copied()),
        )
        .await;
    }
}

impl<'d> DrawTarget for Display<'d> {
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

impl<'d> OriginDimensions for Display<'d> {
    fn size(&self) -> Size {
        Size::new(HEIGHT as u32, WIDTH as u32)
    }
}
