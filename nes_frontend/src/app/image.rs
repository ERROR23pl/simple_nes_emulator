// use std::sync::Arc;

// use egui::{TextureOptions, epaint::*};

// const WIDTH: usize = 256;
// const HEIGHT: usize = 240;

// pub fn test_image() -> ImageData {
//     let buffer = vec![128u8; WIDTH * HEIGHT * 3];
//     let image = ColorImage::from_rgb([WIDTH, HEIGHT], &buffer);
    
//     println!("{}, and {}", buffer.len(), buffer[1]);
//     ImageData::Color(Arc::new(image))
// }

// pub fn update_image(color: [u8; 3], image_data: &mut TextureHandle) {
//     let buffer: Vec<u8> = vec![color; WIDTH * HEIGHT].into_iter().flatten().collect();
//     let image = ColorImage::from_rgb([WIDTH, HEIGHT], &buffer);
//     image_data.set(image, TextureOptions::default());
// }

// pub fn update_image(color: [u8; 3], image_data: &mut TextureHandle, frame: &mut usize) {
//     let buffer: Vec<u8> = vec![color; WIDTH * HEIGHT].into_iter().flatten().collect();
//     let mut image = ColorImage::from_rgb([WIDTH, HEIGHT], &buffer);
    
//     for row in 0..HEIGHT {
//         image[((*frame + row) % WIDTH, row)] = Color32::BLACK;
//     }
    
//     image_data.set(image, TextureOptions::default());
    
//     *frame = (*frame + 1) % WIDTH;
// }