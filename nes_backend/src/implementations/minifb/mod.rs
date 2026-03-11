mod minifb_rendering;

use std::rc::Rc;
use std::cell::RefCell;

use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

use minifb_rendering::*;
use crate::emulator::EmulatorImplementation;
use crate::cartridge::Cartridge;
use crate::rendering;
use crate::rendering::*;
use crate::file_loading::*;
use crate::nes::NES;

type SharedMiniFBScreenBuffer = Rc<RefCell<MiniFBScreenBuffer>>;

pub struct MiniFBEMulator {
    buffer: SharedMiniFBScreenBuffer,
}

impl MiniFBEMulator {
    pub fn new() -> Self {
        Self {
            buffer: Rc::new(RefCell::new(MiniFBScreenBuffer {
                main_screen: [0; _],
                pattern_screen: [0; _],
            })),
        }
    }
}


fn new_window(name: &str, dimensions: Dimensions) -> Window {
    let mut tmp = Window::new(
        name,
        dimensions.width,
        dimensions.height,
        WindowOptions {
            resize: true,
            scale_mode: ScaleMode::AspectRatioStretch,
            scale: Scale::X2,
            ..WindowOptions::default()
        },
    )
    .expect("I have no idea what to do if the window doesn't start so let's just panic.");

    tmp.set_target_fps(60);

    tmp
}

impl EmulatorImplementation for MiniFBEMulator {
    fn run(&mut self) {
        // setup the windows and menus
        let mut emulator_window = new_window("NES emulator", EMULATOR_SCREEN_SIZE);
        let mut debug_window = new_window("NES emulator - Pattern Tables", PATTERN_TABLES_SIZE);        

        emulator_window.set_position(20, 20);
        debug_window.set_position(emulator_window.get_position().0 + 2* 256 + 10, emulator_window.get_position().1);

        // use minifb::Menu;
        // let mut menu = Menu::new("File").unwrap();
        // menu.add_item("load file", 69).build();

        // let _ = emulator_window.add_menu(&menu);

        // waiting for a file
        let file = INesFile::new_from_file_path("./roms/Super Mario Bros. (World).nes").unwrap();
        let mut nes = NES::new(Rc::clone(&self.buffer), Cartridge::new(&file).unwrap());
        self.load_pattern(&mut debug_window, &file);
        while emulator_window.is_open() && !emulator_window.is_key_down(Key::Escape) {
            if emulator_window.is_key_pressed(Key::C, minifb::KeyRepeat::No) {
                nes.clock();
            }            
            
            // update all screens
            self.update_screen(&mut emulator_window);
            self.update_pattern_screen(&mut debug_window);
       }       
    }

    fn insert_cartrigde(&mut self, cartridge: Cartridge) {
        todo!()
    }

    fn reset(&mut self) {
        todo!()
    }
}

// todo: this is extremely retarded. Don't do that
impl MiniFBEMulator {
    // ! you can't do that normally but it's fine with mapper 0
    fn load_pattern(&mut self, window: &mut Window, file: &INesFile) {
        let pattern = file.chr_rom_data();
        
        assert_eq!(pattern.len() * 8, PATTERN_TABLES_SIZE.num_of_pixels() * 2);
        let mut new_buffer = [u32::MAX; PATTERN_TABLES_SIZE.num_of_pixels()];

        // don't worry about it
        const TEMP_PALLETTE: [u8; 4] = [
            0x3F,
            0x06,
            0x09,
            0x02,
        ];

        let mut current = 0;
        for plane in [PatternTable::Left, PatternTable::Right] {
            for row in 0..16 {
                // let row_offset = row * 8 * 16 * 8 * 2;
                let row_offset = row * 8;
                for col in 0..16 {
                    let x_offset = 8 * col;

                    let tile = &pattern[current..(current + 16)];
                    let lbp = &tile[0..8];
                    let mbp = &tile[8..16];
                    
                    for i in 0..8 {
                        let lbp_byte = lbp[i];
                        let mbp_byte = mbp[i];
                        for j in 0..8 {
                            let palette_index = ((lbp_byte >> j) & 0x01) + ((mbp_byte >> j << 1) & 0x02);
                            // new_buffer[plane * 128 + x_offset + (7 - j) + i * EMULATOR_SCREEN_SIZE.width + row_offset] = TEMP_PALLETTE[palette_index as usize];
                            self.buffer.borrow_mut().set_pixel_pattern_table(
                                plane,
                                x_offset + (7 - j),
                                i + row_offset,
                                TEMP_PALLETTE[palette_index as usize],
                            );
                        }
                    }

                    current += 16;
                }
            }
        }
        self.update_pattern_screen(window);
        // window.update_with_buffer(&new_buffer, 256, 128).unwrap();
        // window.update_with_buffer(&new_buffer, 256, 128).unwrap();
    }

    fn update_screen(&mut self, window: &mut Window) {
        let mut new_buffer: [u32; EMULATOR_SCREEN_SIZE.num_of_pixels()] = [0u32.into(); _];

        for (i, n) in self.buffer.borrow().main_screen.iter().enumerate() {
            new_buffer[i] = Pixel::PAL_COLOR[*n as usize].into();
        }

        window.update_with_buffer(
            new_buffer.as_slice(),
            EMULATOR_SCREEN_SIZE.width,
            EMULATOR_SCREEN_SIZE.height
        ).unwrap();
    }
    
    fn update_pattern_screen(&mut self, window: &mut Window) {
        let mut new_buffer: [u32; PATTERN_TABLES_SIZE.num_of_pixels()] = [0u32; _];

        for (i, n) in self.buffer.borrow().pattern_screen.iter().enumerate() {
            new_buffer[i] = Pixel::PAL_COLOR[*n as usize].into();
        }

        window.update_with_buffer(
            new_buffer.as_slice(),
            PATTERN_TABLES_SIZE.width,
            PATTERN_TABLES_SIZE.height
        ).unwrap();
    }
}


impl PixelBuffer for SharedMiniFBScreenBuffer {
    fn get_pixel(&self, x: usize, y: usize) -> NesColor {
        self.borrow().get_pixel(x, y)
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: NesColor) {
        self.borrow_mut().set_pixel(x, y, color)
    }

    fn get(&self, index: usize) -> NesColor {
        self.borrow().get(index)
    }

    fn set(&mut self, index: usize, color: NesColor) {
        self.borrow_mut().set(index, color)
    }

    fn set_pixel_pattern_table(&mut self, pattern_table: PatternTable, x: usize, y: usize, color: NesColor) {
        self.borrow_mut().set_pixel_pattern_table(pattern_table, x, y, color)
    }

    fn get_pixel_pattern_table(&self, pattern_table: PatternTable, x: usize, y: usize) -> NesColor {
        self.borrow().get_pixel_pattern_table(pattern_table, x, y)
    }

    fn render_frame(&mut self) {
        self.borrow_mut().render_frame()
    }
}