pub type NesColor = u8;

pub trait PixelBuffer {
    fn get_pixel(&self, x: usize, y: usize) -> NesColor;
    fn set_pixel(&mut self, x: usize, y: usize, color: NesColor);
    fn get(&self, index: usize) -> NesColor;
    fn set(&mut self, index: usize, color: NesColor);

    fn into_slice(&self) -> &[NesColor];
}

#[derive(Default)]
pub struct DummyBuffer;

impl PixelBuffer for DummyBuffer {
    fn get_pixel(&self, _x: usize, _y: usize) -> NesColor { 0 }
    fn set_pixel(&mut self, _x: usize, _y: usize, _color: NesColor) { }
    fn get(&self, _index: usize) -> NesColor { 0 }
    fn set(&mut self, _index: usize, _color: NesColor) { }
    fn into_slice(&self) -> &[NesColor] { &[0u8; 256 * 240] }
}

#[derive(Clone, Copy)]
pub enum PatternTable {
    Left = 0,
    Right = 1,
}

