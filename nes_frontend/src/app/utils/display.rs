// crate imports
use nes_backend::rendering::*;

// std imports
use std::sync::Arc;

// third-party imports
use egui::{ColorImage, ImageData, TextureHandle, TextureOptions};

pub struct Screen {
    pub handle: TextureHandle,
    // buffer: Vec<u8>,
    pub buffer: BasicPixelBuffer,
}

impl Screen {
    pub fn new(ctx: &egui::Context, name: &str, width: usize, height: usize) -> Self {
        let byte_buffer: Vec<u8> = vec![0; width * height * 3];
        let image = ColorImage::from_rgb([width, height], &byte_buffer);
        let image_data = ImageData::Color(Arc::new(image));
        let handle = ctx.load_texture(name, image_data, TextureOptions::default());
        
        Screen { handle, buffer: BasicPixelBuffer::new(width, height) }
    }

    pub fn update(&mut self) {
        let [width, height] = self.handle.size();

        let byte_buffer: Vec<u8> = self.buffer.into_slice().iter()
            .map(|c| PAL_COLOR[*c as usize])
            .flatten()
            .collect();

        let image = ColorImage::from_rgb([width, height], &byte_buffer);
        
        self.handle.set(image, TextureOptions::default());       
    }

    // pub fn update_with_pixel_buffer<P: PixelBuffer>(&mut self, buffer: &P) {
    //     let [width, height] = self.handle.size();

    //     let byte_buffer: Vec<u8> = buffer.into_slice().iter()
    //         .map(|c| PAL_COLOR[*c as usize])
    //         .flatten()
    //         .collect();

    //     let image = ColorImage::from_rgb([width, height], &byte_buffer);
        
    //     self.handle.set(image, TextureOptions::default());
    // }
}

// // * debuing functions
// impl Screen {
//     pub fn color_flicker(&mut self, color: [u8; 3], frame_count: &mut usize) {
//         let [width, height] = self.handle.size();

//         self.buffer = vec![color; width * height].into_iter().flatten().collect();
//         let mut image = ColorImage::from_rgb([width, height], &self.buffer);
        
//         for row in 0..height {
//             image[((*frame_count + row) % width, row)] = Color32::BLACK;
//         }
        
//         self.handle.set(image, TextureOptions::default());
        
//         *frame_count = (*frame_count + 1) % width;
//     }
// }


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

// color scheme by Bisqit
// todo: link the source
pub const PAL_COLOR: [[u8; 3]; 0x40] = [
    [0x54, 0x54, 0x54],
    [0x00, 0x1E, 0x74],
    [0x08, 0x10, 0x90],
    [0x30, 0x00, 0x88],
    [0x44, 0x00, 0x64],
    [0x5C, 0x00, 0x30],
    [0x54, 0x04, 0x00],
    [0x3C, 0x18, 0x00],
    [0x20, 0x2A, 0x00],
    [0x08, 0x3A, 0x00],
    [0x00, 0x40, 0x00],
    [0x00, 0x3C, 0x00],
    [0x00, 0x32, 0x3C],
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00],

    [0x98, 0x96, 0x98],
    [0x08, 0x4C, 0xC4],
    [0x30, 0x32, 0xEC],
    [0x5C, 0x1E, 0xE4],
    [0x88, 0x14, 0xB0],
    [0xA0, 0x14, 0x64],
    [0x98, 0x22, 0x20],
    [0x78, 0x3C, 0x00],
    [0x54, 0x5A, 0x00],
    [0x28, 0x72, 0x00],
    [0x08, 0x7C, 0x00],
    [0x00, 0x76, 0x28],
    [0x00, 0x66, 0x78],
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00],

    [0xEC, 0xEE, 0xEC],
    [0x4C, 0x9A, 0xEC],
    [0x78, 0x7C, 0xEC],
    [0xB0, 0x62, 0xEC],
    [0xE4, 0x54, 0xEC],
    [0xEC, 0x58, 0xB4],
    [0xEC, 0x6A, 0x64],
    [0xD4, 0x88, 0x20],
    [0xA0, 0xAA, 0x00],
    [0x74, 0xC4, 0x00],
    [0x4C, 0xD0, 0x20],
    [0x38, 0xCC, 0x6C],
    [0x38, 0xB4, 0xCC],
    [0x3C, 0x3C, 0x3C],
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00],

    [0xEC, 0xEE, 0xEC],
    [0xA8, 0xCC, 0xEC],
    [0xBC, 0xBC, 0xEC],
    [0xD4, 0xB2, 0xEC],
    [0xEC, 0xAE, 0xEC],
    [0xEC, 0xAE, 0xD4],
    [0xEC, 0xB4, 0xB0],
    [0xE4, 0xC4, 0x90],
    [0xCC, 0xD2, 0x78],
    [0xB4, 0xDE, 0x78],
    [0xA8, 0xE2, 0x90],
    [0x98, 0xE2, 0xB4],
    [0xA0, 0xD6, 0xE4],
    [0xA0, 0xA2, 0xA0],
    [0x00, 0x00, 0x00],
    [0x00, 0x00, 0x00],
];
