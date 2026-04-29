use crate::{
    bit_operations::BitManipulation,
    cartridge::Cartridge,
    ppu::PPU,
    rendering::PixelBuffer
};

impl<P: PixelBuffer> PPU<P> {
    pub fn clock(&mut self, cartridge: &mut Cartridge) {
        const CYCLE_NUMBER: u16 = 341;
        const SCANLINE_NUMBER: i16 = 261;

        // As we progress through scanlines and cycles, the PPU is effectively
        // a state machine going through the motions of fetching background 
        // information and sprite information, compositing them into a pixel
        // to be output.

        if matches!(self.scanline, -1..240) { // visible scanline
            if self.scanline == 0 && self.cycle == 0 {
                // "Odd Frame" cycle skip
                self.cycle = 1;
            }

            if self.scanline == -1 && self.cycle == 1 {
                // Effectively start of new frame, so clear vertical blank flag
                self.ppu_controls.status.set_vertical_blank(false);
            }

            if matches!(self.cycle, 2..258 | 321..338) {
                self.update_shifters();
                
                // In these cycles we are collecting and working with visible data
                // The "shifters" have been preloaded by the end of the previous
                // scanline with the data for the start of this scanline. Once we
                // leave the visible region, we go dormant until the shifters are
                // preloaded for the next scanline.

                // Fortunately, for background rendering, we go through a fairly
                // repeatable sequence of events, every 2 clock cycles.
                match (self.cycle - 1) % 8 {
                    0 => {
                        // Load the current background tile pattern and attributes into the "shifter"
                        self.load_background_shifters();

                        // Fetch the next background tile ID
                        // "(vram_addr.reg & 0x0FFF)" : Mask to 12 bits that are relevant
                        // "| 0x2000"                 : Offset into nametable space on PPU address bus
                        self.bg_next_tile_id = self.read(
                            0x2000 + (u16::from(self.ppu_controls.vram_address) & 0x0FFF),
                            cartridge
                        );

                        // Explanation:
                        // The bottom 12 bits of the loopy register provide an index into
                        // the 4 nametables, regardless of nametable mirroring configuration.
                        // nametable_y(1) nametable_x(1) coarse_y(5) coarse_x(5)
                        //
                        // Consider a single nametable is a 32x32 array, and we have four of them
                        //   0                1
                        // 0 +----------------+----------------+
                        //   |                |                |
                        //   |                |                |
                        //   |    (32x32)     |    (32x32)     |
                        //   |                |                |
                        //   |                |                |
                        // 1 +----------------+----------------+
                        //   |                |                |
                        //   |                |                |
                        //   |    (32x32)     |    (32x32)     |
                        //   |                |                |
                        //   |                |                |
                        //   +----------------+----------------+
                        //
                        // This means there are 4096 potential locations in this array, which 
                        // just so happens to be 2^12
                    },

                    2 => {
                        // Fetch the next background tile attribute. OK, so this one is a bit
                        // more involved :P

                        // Recall that each nametable has two rows of cells that are not tile 
                        // information, instead they represent the attribute information that
                        // indicates which palettes are applied to which area on the screen.
                        // Importantly (and frustratingly) there is not a 1 to 1 correspondance
                        // between background tile and palette. Two rows of tile data holds
                        // 64 attributes. Therfore we can assume that the attributes affect
                        // 8x8 zones on the screen for that nametable. Given a working resolution
                        // of 256x240, we can further assume that each zone is 32x32 pixels
                        // in screen space, or 4x4 tiles. Four system palettes are allocated
                        // to background rendering, so a palette can be specified using just
                        // 2 bits. The attribute byte therefore can specify 4 distinct palettes.
                        // Therefore we can even further assume that a single palette is
                        // applied to a 2x2 tile combination of the 4x4 tile zone. The very fact
                        // that background tiles "share" a palette locally is the reason why
                        // in some games you see distortion in the colours at screen edges.

                        // As before when choosing the tile ID, we can use the bottom 12 bits of
                        // the loopy register, but we need to make the implementation "coarser"
                        // because instead of a specific tile, we want the attribute byte for a 
                        // group of 4x4 tiles, or in other words, we divide our 32x32 address
                        // by 4 to give us an equivalent 8x8 address, and we offset this address
                        // into the attribute section of the target nametable.

                        // Reconstruct the 12 bit loopy address into an offset into the
                        // attribute memory

                        // "(vram_addr.coarse_x >> 2)"        : integer divide coarse x by 4, 
                        //                                      from 5 bits to 3 bits
                        // "((vram_addr.coarse_y >> 2) << 3)" : integer divide coarse y by 4, 
                        //                                      from 5 bits to 3 bits,
                        //                                      shift to make room for coarse x
                        const ATTRIBUTE_MEMORY_OFFSET: u16 = 32 * 30;
                        const NAMETABLE_MEMORY_OFFSET: u16 = 0x2000;
                        let (_, nametable_y, nametable_x, coarse_y, coarse_x) = &self.ppu_controls.vram_address.dissolve_u16();
                        
                        self.bg_next_tile_attrib = self.read(
                            NAMETABLE_MEMORY_OFFSET + ATTRIBUTE_MEMORY_OFFSET + (
                                (nametable_y << 11) | (nametable_x << 10) | ((coarse_y >> 2) << 3) | (coarse_x >> 2)
                            ),
                            cartridge
                        );

                        // Result so far: YX00 00yy yxxx

                        // All attribute memory begins at 0x03C0 within a nametable, so OR with
                        // result to select target nametable, and attribute byte offset. Finally
                        // OR with 0x2000 to offset into nametable address space on PPU bus.

                        // Right we've read the correct attribute byte for a specified address,
                        // but the byte itself is broken down further into the 2x2 tile groups
                        // in the 4x4 attribute zone.

                        // The attribute byte is assembled thus: BR(76) BL(54) TR(32) TL(10)
                        //
                        // +----+----+			    +----+----+
                        // | TL | TR |			    | ID | ID |
                        // +----+----+ where TL =   +----+----+
                        // | BL | BR |			    | ID | ID |
                        // +----+----+			    +----+----+
                        //
                        // Since we know we can access a tile directly from the 12 bit address, we
                        // can analyse the bottom bits of the coarse coordinates to provide us with
                        // the correct offset into the 8-bit word, to yield the 2 bits we are
                        // actually interested in which specifies the palette for the 2x2 group of
                        // tiles. We know if "coarse y % 4" < 2 we are in the top half else bottom half.
                        // Likewise if "coarse x % 4" < 2 we are in the left half else right half.
                        let top = coarse_y % 4 < 2;
                        let left = coarse_x % 4 < 2;

                        self.bg_next_tile_attrib >>= match (top, left) {
                            // TL, top-left
                            (true, true) => 0,
                            // TR, top-right
                            (true, false) => 2,
                            // BL, bottom-left
                            (false, true) => 4,
                            // BR, bottom-right
                            (false, false) => 6,
                        };

                        // only interested in last 2 bits                        
                        self.bg_next_tile_attrib &= 0x03;
                        
                        // Ultimately we want the bottom two bits of our attribute word to be the
                        // palette selected. So shift as required...
                        // if (coarse_y & 0x02) != 0 { self.bg_next_tile_attrib >>= 4 };
                        // if (coarse_x & 0x02) != 0 { self.bg_next_tile_attrib >>= 2 };
                        // self.bg_next_tile_attrib &= 0x03;
                    },

                    4 => {
                        // Fetch the next background tile LSB bit plane from the pattern memory
                        // The Tile ID has been read from the nametable. We will use this id to 
                        // index into the pattern memory to find the correct sprite (assuming
                        // the sprites lie on 8x8 pixel boundaries in that memory, which they do
                        // even though 8x16 sprites exist, as background tiles are always 8x8).
                        //
                        // Since the sprites are effectively 1 bit deep, but 8 pixels wide, we 
                        // can represent a whole sprite row as a single byte, so offsetting
                        // into the pattern memory is easy. In total there is 8KB so we need a 
                        // 13 bit address.

                        // "(control.pattern_background << 12)"  : the pattern memory selector 
                        //                                         from control register, either 0K
                        //                                         or 4K offset
                        // "((uint16_t)bg_next_tile_id << 4)"    : the tile id multiplied by 16, as
                        //                                         2 lots of 8 rows of 8 bit pixels
                        // "(vram_addr.fine_y)"                  : Offset into which row based on
                        //                                         vertical scroll offset
                        // "+ 0"                                 : Mental clarity for plane offset
                        // Note: No PPU address bus offset required as it starts at 0x0000
                        self.bg_next_tile_lsb = self.read(
                            ((self.ppu_controls.control.pattern_background() as u16) << 12) +
                            ((self.bg_next_tile_id as u16) << 4) +
                            (self.ppu_controls.vram_address.fine_y() as u16),
                            cartridge
                        );
                    },
                    
                    6 => {
                        // Fetch the next background tile MSB bit plane from the pattern memory
                        // This is the same as above, but has a +8 offset to select the next bit plane
                        self.bg_next_tile_msb = self.read(
                            ((self.ppu_controls.control.pattern_background() as u16) << 12) +
                            ((self.bg_next_tile_id as u16) << 4) +
                            (self.ppu_controls.vram_address.fine_y() as u16) + 8,
                            cartridge
                        );
                    },

                    7 => {
                        // Increment the background tile "pointer" to the next tile horizontally
                        // in the nametable memory. Note this may cross nametable boundaries which
                        // is a little complex, but essential to implement scrolling
                        self.increment_scroll_x();
                    },

                    1 | 3 | 5 => { /* on these frames we do nothing */ },

                    8.. => unreachable!("Any number modulo 8 can't be bigger than 8"),
                }
            }

            // End of a visible scanline, so increment downwards...
            if self.cycle == 256 {
                self.increment_scroll_y();
            }

            //...and reset the x position
            if self.cycle == 257 {
                self.load_background_shifters();
                self.transfer_address_x();
            }

            // Superfluous reads of tile id at end of scanline
            if self.cycle == 338 || self.cycle == 340 {
                self.bg_next_tile_id = self.read(0x2000 | (u16::from(self.ppu_controls.vram_address) & 0x0FFF), cartridge);
            }

            if self.scanline == -1 && matches!(self.cycle, 280..305) {
                // End of vertical blank period so reset the Y address ready for rendering
                self.transfer_address_y();
            }
        }

        if self.scanline == 240 {
            // Post Render Scanline - Do Nothing!
        }

        if matches!(self.scanline, 241..261) {
            if self.scanline == 241 && self.cycle == 1 {
                // Effectively end of frame, so set vertical blank flag
                self.ppu_controls.status.set_vertical_blank(true);

                // If the control register tells us to emit a NMI when
                // entering vertical blanking period, do it! The CPU
                // will be informed that rendering is complete so it can
                // perform operations with the PPU knowing it wont
                // produce visible artefacts
                if self.ppu_controls.control.enable_nmi() {
                    self.nmi = true;
                }
            }
        }

        // Composition - We now have background pixel information for this cycle
        // At this point we are only interested in background

        let mut bg_pixel: u8 = 0x00;   // The 2-bit pixel to be rendered
        let mut bg_palette: u8 = 0x00; // The 3-bit index of the palette the pixel indexes
        
        // We only render backgrounds if the PPU is enabled to do so. Note if 
        // background rendering is disabled, the pixel and palette combine
        // to form 0x00. This will fall through the colour tables to yield
        // the current background colour in effect
        if self.ppu_controls.mask.render_background() && matches!(self.cycle, 1..=256) && matches!(self.scanline, 0..240) {
            // Handle Pixel Selection by selecting the relevant bit
            // depending upon fine x scolling. This has the effect of
            // offsetting ALL background rendering by a set number
            // of pixels, permitting smooth scrolling
            let bit_mux: u16 = 0x8000 >> self.ppu_controls.fine_x;

            // Select Plane pixels by extracting from the shifter 
            // at the required location. 
            let p0_pixel: u8 = ((self.bg_shifter_pattern_lo & bit_mux) > 0) as u8;
            let p1_pixel: u8 = ((self.bg_shifter_pattern_hi & bit_mux) > 0) as u8;

            // Combine to form pixel index
            bg_pixel = (p1_pixel << 1) | p0_pixel;

            // Get palette
            let bg_pal0: u8 = ((self.bg_shifter_attrib_lo & bit_mux) > 0) as u8;
            let bg_pal1: u8 = ((self.bg_shifter_attrib_hi & bit_mux) > 0) as u8;
            bg_palette = (bg_pal1 << 1) | bg_pal0;

            // Now we have a final pixel colour, and a palette for this cycle
            // of the current scanline. Let's at long last, draw that ^&%*er :P

            let color = self.get_color_value_from_pallet_ram(bg_palette, bg_pixel);
            self.screen.set_pixel((self.cycle - 1) as usize, self.scanline as usize, color);
        }    

        // Advance renderer - it never stops, it's relentless
        self.cycle += 1;

        if self.cycle >= CYCLE_NUMBER {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline >= SCANLINE_NUMBER {
                self.scanline = -1;
                self.frame_complete = true;
            }
        }
    }
    
    // ==============================================================================
    // Increment the background tile "pointer" one tile/column horizontally
    pub fn increment_scroll_y(&mut self) {
        // Incrementing vertically is more complicated. The visible nametable
        // is 32x30 tiles, but in memory there is enough room for 32x32 tiles.
        // The bottom two rows of tiles are in fact not tiles at all, they
        // contain the "attribute" information for the entire table. This is
        // information that describes which palettes are used for different 
        // regions of the nametable.
        
        // In addition, the NES doesnt scroll vertically in chunks of 8 pixels
        // i.e. the height of a tile, it can perform fine scrolling by using
        // the fine_y component of the register. This means an increment in Y
        // first adjusts the fine offset, but may need to adjust the whole
        // row offset, since fine_y is a value 0 to 7, and a row is 8 pixels high

        // Ony if rendering is enabled
        if self.ppu_controls.mask.render_background() || self.ppu_controls.mask.render_sprites() {
            // If possible, just increment the fine y offset
            if self.ppu_controls.vram_address.fine_y() < 7 {
                self.ppu_controls.vram_address.set_fine_y(self.ppu_controls.vram_address.fine_y() + 1);
                return;
            }

            // If we have gone beyond the height of a row, we need to
            // increment the row, potentially wrapping into neighbouring
            // vertical nametables. Dont forget however, the bottom two rows
            // do not contain tile information. The coarse y offset is used
            // to identify which row of the nametable we want, and the fine
            // y offset is the specific "scanline"

            // Reset fine y offset
            self.ppu_controls.vram_address.set_fine_y(0);

            // Check if we need to swap vertical nametable targets
            if self.ppu_controls.vram_address.coarse_y() == 29 {
                // We do, so reset coarse y offset
                self.ppu_controls.vram_address.set_coarse_y(0);
                // And flip the target nametable bit
                self.ppu_controls.vram_address.set_nametable_y(!self.ppu_controls.vram_address.nametable_y());
            }
            else if self.ppu_controls.vram_address.coarse_y() == 31 {
                // In case the pointer is in the attribute memory, we
                // just wrap around the current nametable
                self.ppu_controls.vram_address.set_coarse_y(0);
            }
            else {
                // None of the above boundary/wrapping conditions apply
                // so just increment the coarse y offset
                self.ppu_controls.vram_address.set_coarse_y(self.ppu_controls.vram_address.coarse_y() + 1);
            }
        }
    }

    // ==============================================================================
    // Increment the background tile "pointer" one scanline vertically       
    fn increment_scroll_x(&mut self) {
        // Note: pixel perfect scrolling horizontally is handled by the 
        // data shifters. Here we are operating in the spatial domain of 
        // tiles, 8x8 pixel blocks.
        
        // Ony if rendering is enabled
        if self.ppu_controls.mask.render_background() || self.ppu_controls.mask.render_sprites() {
            // A single name table is 32x30 tiles. As we increment horizontally
            // we may cross into a neighbouring nametable, or wrap around to
            // a neighbouring nametable
            if self.ppu_controls.vram_address.coarse_x() == 31 {
                // Leaving nametable so wrap address round
                self.ppu_controls.vram_address.set_coarse_x(0);
                // Flip target nametable bit
                self.ppu_controls.vram_address.set_nametable_x(!self.ppu_controls.vram_address.nametable_x());
            }
            else {
                // Staying in current nametable, so just increment
                self.ppu_controls.vram_address.set_coarse_x(self.ppu_controls.vram_address.coarse_x() + 1);
            }
        }
    }

    // ==============================================================================
    // Transfer the temporarily stored horizontal nametable access information
    // into the "pointer". Note that fine x scrolling is not part of the "pointer"
    // addressing mechanism    
    fn transfer_address_x(&mut self) {
        // Ony if rendering is enabled
        if self.ppu_controls.mask.render_background() || self.ppu_controls.mask.render_sprites() {
            self.ppu_controls.vram_address.set_nametable_x(self.ppu_controls.tram_address.nametable_x());
            self.ppu_controls.vram_address.set_coarse_x(self.ppu_controls.tram_address.coarse_x());
        }
    }
    
    // ==============================================================================
    // Transfer the temporarily stored vertical nametable access information
    // into the "pointer". Note that fine y scrolling is part of the "pointer"
    // addressing mechanism 
    fn transfer_address_y(&mut self) {
        // Ony if rendering is enabled
        if self.ppu_controls.mask.render_background() || self.ppu_controls.mask.render_sprites() {
            self.ppu_controls.vram_address.set_fine_y(self.ppu_controls.tram_address.fine_y());
            self.ppu_controls.vram_address.set_nametable_y(self.ppu_controls.tram_address.nametable_y());
            self.ppu_controls.vram_address.set_coarse_y(self.ppu_controls.tram_address.coarse_y());
        }
    }

    // ==============================================================================
    // Prime the "in-effect" background tile shifters ready for outputting next
    // 8 pixels in scanline. 
    fn load_background_shifters(&mut self) {	
        // Each PPU update we calculate one pixel. These shifters shift 1 bit along
        // feeding the pixel compositor with the binary information it needs. Its
        // 16 bits wide, because the top 8 bits are the current 8 pixels being drawn
        // and the bottom 8 bits are the next 8 pixels to be drawn. Naturally this means
        // the required bit is always the MSB of the shifter. However, "fine x" scrolling
        // plays a part in this too, whcih is seen later, so in fact we can choose
        // any one of the top 8 bits.
        self.bg_shifter_pattern_lo = (self.bg_shifter_pattern_lo & 0xFF00) | self.bg_next_tile_lsb as u16;
        self.bg_shifter_pattern_hi = (self.bg_shifter_pattern_hi & 0xFF00) | self.bg_next_tile_msb as u16;

        // Attribute bits do not change per pixel, rather they change every 8 pixels
        // but are synchronised with the pattern shifters for convenience, so here
        // we take the bottom 2 bits of the attribute word which represent which 
        // palette is being used for the current 8 pixels and the next 8 pixels, and 
        // "inflate" them to 8 bit words.
        self.bg_shifter_attrib_lo  = (self.bg_shifter_attrib_lo & 0xFF00) | (if self.bg_next_tile_attrib.nth_bit(0) == 1 { 0xFF } else { 0x00 });
        self.bg_shifter_attrib_hi  = (self.bg_shifter_attrib_hi & 0xFF00) | (if self.bg_next_tile_attrib.nth_bit(1) == 1 { 0xFF } else { 0x00 });
    }

    // ==============================================================================
    // Every cycle the shifters storing pattern and attribute information shift
    // their contents by 1 bit. This is because every cycle, the output progresses
    // by 1 pixel. This means relatively, the state of the shifter is in sync
    // with the pixels being drawn for that 8 pixel section of the scanline.

    // All but 1 of the secanlines is visible to the user. The pre-render scanline
    // at -1, is used to configure the "shifters" for the first visible scanline, 0.
    fn update_shifters(&mut self) {
        if self.ppu_controls.mask.render_background() {
            // Shifting background tile pattern row
            self.bg_shifter_pattern_lo <<= 1;
            self.bg_shifter_pattern_hi <<= 1;

            // Shifting palette attributes by 1
            self.bg_shifter_attrib_lo <<= 1;
            self.bg_shifter_attrib_hi <<= 1;
        }
    }
}

// macro_rules! load_background_shifters {
//     ($self:ident) => {
//         $self.bg_shifter_pattern_lo = ($self.bg_shifter_pattern_lo & 0xFF00) | $self.bg_next_tile_lsb as u16;
//         $self.bg_shifter_pattern_hi = ($self.bg_shifter_pattern_hi & 0xFF00) | $self.bg_next_tile_msb as u16;
//         $self.bg_shifter_attrib_lo  = ($self.bg_shifter_attrib_lo & 0xFF00) | (if $self.bg_next_tile_attrib & 0b01 != 0 { 0xFF } else { 0x00 });
//         $self.bg_shifter_attrib_hi  = ($self.bg_shifter_attrib_hi & 0xFF00) | (if $self.bg_next_tile_attrib & 0b10 != 0 { 0xFF } else { 0x00 });
//     };
// }

// macro_rules! update_shifters {
//     ($self:ident) => {
//         if $self.ppu_controls.mask.render_background() {
//             // Shifting background tile pattern row
//             $self.bg_shifter_pattern_lo <<= 1;
//             $self.bg_shifter_pattern_hi <<= 1;

//             // Shifting palette attributes by 1
//             $self.bg_shifter_attrib_lo <<= 1;
//             $self.bg_shifter_attrib_hi <<= 1;
//         }        
//     };
// }

