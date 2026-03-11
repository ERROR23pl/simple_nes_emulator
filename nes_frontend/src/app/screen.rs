use std::sync::Arc;

use egui::{
    Color32, ColorImage, ImageData, TextureHandle, TextureOptions
};

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
    
    pub fn update_image(&mut self, color: [u8; 3], frame: &mut usize) {
        let [width, height] = self.handle.size();

        self.buffer = vec![color; width * height].into_iter().flatten().collect();
        let mut image = ColorImage::from_rgb([width, height], &self.buffer);
        
        for row in 0..height {
            image[((*frame + row) % width, row)] = Color32::BLACK;
        }
        
        self.handle.set(image, TextureOptions::default());
        
        *frame = (*frame + 1) % width;
    }
}