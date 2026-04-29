// crate imports
use nes_backend::rendering::{self, *};

// std imports
use std::sync::Arc;

// third-party imports
use egui::{Color32, ColorImage, ImageData, TextureHandle, TextureOptions };

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

    pub fn update_with_pixel_buffer<P: PixelBuffer>(&mut self, buffer: &P) {
        let [width, height] = self.handle.size();

        self.buffer = buffer.into_slice().iter()
            .map(|c| rendering::PAL_COLOR[*c as usize])
            .flatten()
            .collect();

        let image = ColorImage::from_rgb([width, height], &self.buffer);
        
        self.handle.set(image, TextureOptions::default());
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


pub struct BasicPixelBuffer {
    width: usize,
    height: usize,
    buffer: Vec<NesColor>,
    ready_to_render: bool,
}

impl BasicPixelBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0; width * height],
            ready_to_render: false,
        }
    }
    
    fn coor_to_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn index_to_coor(&self, index: usize) -> (usize, usize) {
        (index % self.height, index / self.height)
    }
}

impl PixelBuffer for BasicPixelBuffer {
    fn get_pixel(&self, x: usize, y: usize) -> NesColor {
        self.buffer[self.coor_to_index(x, y)]
    }
    
    fn set_pixel(&mut self, x: usize, y: usize, color: NesColor) {
        let coor = self.coor_to_index(x, y);
        self.buffer[coor] = color;
    }

    fn get(&self, index: usize) -> NesColor {
        self.buffer[index]       
    }

    fn set(&mut self, index: usize, color: NesColor) {
        self.buffer[index] = color;
    }

    fn into_slice(&self) -> &[NesColor] {
        &self.buffer
    }
}

