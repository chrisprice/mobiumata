use smart_leds::RGB8;

pub struct Compressor {
    max: usize,
}

impl Compressor {
    pub fn new(max: usize) -> Self {
        Compressor { max }
    }
    
    pub fn calculate(&self, data: impl Iterator<Item = RGB8>) -> Inner {
        let mut sum = 0;
        
        for color in data {
            sum += color.r as usize + color.g as usize + color.b as usize;
        }
        
        let ratio = sum as f32 / self.max as f32;
        
        if ratio < 1.0 {
            Inner { ratio: 1.0, sum }
        } else {
            Inner { ratio, sum }
        }
    }
}

pub struct Inner {
    pub ratio: f32,
    pub sum: usize,
}

impl Inner {
    pub fn compress<T: Iterator<Item = RGB8>>(&self, data: T) -> InnerInner<T> {
        InnerInner {
            ratio: self.ratio,
            iterator: data,
        }
    }
}

pub struct InnerInner<T: Iterator<Item = RGB8>> {
    ratio: f32,
    iterator: T,
}

impl <T: Iterator<Item = RGB8>> Iterator for InnerInner<T> {
    type Item = RGB8;
    
    fn next(&mut self) -> Option<Self::Item> {
        let color = self.iterator.next()?;

        let r = (color.r as f32 / self.ratio) as u8;
        let g = (color.g as f32 / self.ratio) as u8;
        let b = (color.b as f32 / self.ratio) as u8;

        Some(RGB8 { r, g, b })
    }
}
