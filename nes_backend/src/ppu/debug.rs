use super::*;

impl PPU {
    fn set_pixel_pattern_table<P: PixelBuffer>(
        &mut self,
        screen: &mut P,
        pattern_table: PatternTable,
        x: usize,
        y: usize,
        color: NesColor
    ) {
        let pattern_shift = (pattern_table as usize) * (256 / 2);
        screen.set(x + pattern_shift + y * 256, color);
    }

    pub fn render_debug_pattern_table<P: PixelBuffer>(
        &mut self,
        screen: &mut P,
        pattern_table_side: PatternTable,
        pallete_id: u8,
        cartridge: &Cartridge
    ) {
        // locate the tile in the pattern table
        const SINGLE_PATTERN_TABLE_SIZE: u16 = 0x1000;
        for (tile_y, tile_x) in iproduct!(0..16, 0..16) {
            let offset = (pattern_table_side as u16) * SINGLE_PATTERN_TABLE_SIZE + tile_y * 256 + tile_x * 16;

            // each tile is 8x8 pixels. Rows are stored one after another.
            for row in 0..8 {
                // each row is 2 bytes stored one after another
                let mut tile_lsb = self.read(offset + row + 0, cartridge); 
                let mut tile_msb = self.read(offset + row + 8, cartridge); 

                for col in 0..8 {
                    // pixels are encoded by 2 bits
                    let pixel: u8 = (tile_lsb & 0x01) | ((tile_msb & 0x01) << 1);
                    tile_lsb >>= 1;
                    tile_msb >>= 1;

                    self.set_pixel_pattern_table(
                        screen,
                        pattern_table_side,
                        (tile_x * 8 + (7 - col)) as usize, // `7 - col` since we read it in reverse order
                        (tile_y * 8 + row) as usize,
                        self.get_color_value_from_pallet_ram(pallete_id, pixel)
                    );
                }
            }
        }
    }
}