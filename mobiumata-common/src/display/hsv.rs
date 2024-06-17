use embedded_graphics::pixelcolor::Rgb888;
use smart_leds::hsv::{hsv2rgb, Hsv};


// struct Color

// pub fn hsv(hue: u8, sat: u8, val: u8) -> Rgb888 {
//     let rgb = hsv2rgb(Hsv {
//         hue,
//         sat,
//         val: (val as u16 * (BRIGHTNESS as u16 + 1) / 256) as u8,
//     });
//     Rgb888::new(rgb.r, rgb.g, rgb.b)
// }