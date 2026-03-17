use std::sync::Arc;

use egui::{ Color32, ColorImage, ImageData, TextureHandle, TextureOptions };

pub struct Screen {
    pub handle: TextureHandle,
    buffer: Vec<u8>,
}

impl Screen {
    pub fn new(ctx: &egui::Context, name: &str, width: usize, height: usize) -> Self {
        let buffer: Vec<u8> = vec![0; width * height * 3];
        let image = ColorImage::from_rgb([width, height], &buffer);
        let image_data = ImageData::Color(Arc::new(image));
        let handle = ctx.load_texture(name, image_data, TextureOptions::default());
        
        Screen { handle, buffer }
    }
}

// * debuing functions
impl Screen {
    pub fn color_flicker(&mut self, color: [u8; 3], frame_count: &mut usize) {
        let [width, height] = self.handle.size();

        self.buffer = vec![color; width * height].into_iter().flatten().collect();
        let mut image = ColorImage::from_rgb([width, height], &self.buffer);
        
        for row in 0..height {
            image[((*frame_count + row) % width, row)] = Color32::BLACK;
        }
        
        self.handle.set(image, TextureOptions::default());
        
        *frame_count = (*frame_count + 1) % width;
    }
}

use nes_backend::rendering::*;

pub struct DummyCanvas;

impl Default for DummyCanvas {
    fn default() -> Self {
        Self {  }
    }
}


impl PixelBuffer for DummyCanvas {
    fn get_pixel(&self, x: usize, y: usize) -> NesColor {
        todo!()
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: NesColor) {
        todo!()
    }

    fn get(&self, index: usize) -> NesColor {
        todo!()
    }

    fn set(&mut self, index: usize, color: NesColor) {
        todo!()
    }

    fn get_pixel_pattern_table(&self, pattern_table: PatternTable, x: usize, y: usize) -> NesColor {
        todo!()
    }

    fn set_pixel_pattern_table(&mut self, pattern_table: PatternTable, x: usize, y: usize, color: NesColor) {
        todo!()
    }

    fn render_frame(&mut self) {
        todo!()
    }
}