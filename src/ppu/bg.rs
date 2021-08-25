use super::*;
use crate::bus::*;
use crate::data_unit::*;
use crate::scanline::*;

const NAME_TABLE_FIRST: Word = Word::new(0x2000u16);
const ATTR_TABLE_FIRST: Word = Word::new(0x23C0u16);
const TILE_HEIGHT: Byte = Byte::new(8);

#[derive(Debug, Copy, Clone, Default)]
pub(super) struct BgPattern {
    low: Word,
    high: Word,
}

#[derive(Debug, Copy, Clone, Default)]
pub(super) struct BgPatternAttr {
    low: Byte,
    high: Byte,
    low_latch: bool,
    high_latch: bool,
}

pub(super) trait BgProcessor: Bus {
    fn process(&mut self, ppu: &mut Ppu, scan: Scan) -> Scan {
        let mut scan = scan;

        match scan.dot {
            1 => {
                ppu.background_shift();
                // no shift reloading
                ppu.bg_addr = NAME_TABLE_FIRST | ppu.v.name_table_address_index();
                if let Scanline::Pre = scan.scanline {
                    // End VBLANK
                    ppu.status
                        .remove(Status::VBLANK | Status::SPRITE_ZERO_HIT | Status::SPRITE_OVERFLOW);
                }
            }
            dot @ 2..=255 | dot @ 322..=336 => {
                ppu.background_shift();

                // tile shift
                match dot % 8 {
                    // Fetch nametable byte : step 1
                    1 => {
                        ppu.bg_addr = NAME_TABLE_FIRST | ppu.v.name_table_address_index();
                        ppu.background_reload_shift();
                    }
                    // Fetch nametable byte : step 2
                    2 => {
                        ppu.nt_latch = self.read(ppu.bg_addr);
                    }
                    // Fetch attribute table byte : step 1
                    3 => {
                        ppu.bg_addr = ATTR_TABLE_FIRST | ppu.v.attribute_address_index();
                    }
                    // Fetch attribute table byte : step 2
                    4 => {
                        ppu.at_latch = self.read(ppu.bg_addr);
                        if ppu.v.coarse_x_scroll().nth(0) == 1 {
                            ppu.at_latch >>= 1
                        }
                        if ppu.v.coarse_y_scroll().nth(0) == 1 {
                            ppu.at_latch >>= 3
                        }
                    }
                    // Fetch tile bitmap low byte : step 1
                    5 => {
                        let base: Word = if ppu.controller.contains(Controller::BG_TABLE_ADDR) {
                            0x1000u16
                        } else {
                            0x0000u16
                        }
                        .into();
                        let index = ppu.nt_latch * TILE_HEIGHT * 1;
                        ppu.bg_addr = base + index + ppu.v.fine_y_scroll();
                    }
                    // Fetch tile bitmap low byte : step 2
                    6 => {
                        ppu.bg.low = self.read(ppu.bg_addr).into();
                    }
                    // Fetch tile bitmap high byte : step 1
                    7 => {
                        ppu.bg_addr += <Byte as Into<Word>>::into(TILE_HEIGHT);
                    }
                    // Fetch tile bitmap high byte : step 2
                    0 => {
                        ppu.bg.high = self.read(ppu.bg_addr).into();
                        if ppu.mask.contains(Mask::RENDER_ENABLED) {
                            ppu.incr_coarse_x();
                        }
                    }
                    _ => panic!(),
                }
            }
            256 => {
                ppu.background_shift();
                ppu.bg.high = self.read(ppu.bg_addr).into();
                if ppu.mask.contains(Mask::RENDER_ENABLED) {
                    ppu.incr_y();
                }
            }
            257 => {
                ppu.background_reload_shift();
                if ppu.mask.contains(Mask::RENDER_ENABLED) {
                    ppu.copy_x();
                }
            }
            279..=304 => {
                if let Scanline::Pre = scan.scanline {
                    if ppu.mask.contains(Mask::RENDER_ENABLED) {
                        ppu.copy_y();
                    }
                }
            }
            320 => {
                // no shift reloading
                ppu.bg_addr = NAME_TABLE_FIRST | ppu.v.name_table_address_index();
            }
            // Unused name table fetches
            337 | 339 => {
                ppu.bg_addr = NAME_TABLE_FIRST | ppu.v.name_table_address_index();
            }
            338 | 340 => {
                ppu.nt_latch = self.read(ppu.bg_addr);
            }
            341 => {
                if ppu.mask.contains(Mask::RENDER_ENABLED) && scan.frames % 2 == 0 {
                    // Skip 0 cycle on visible frame
                    scan.skip();
                }
            }
            _ => {}
        }

        scan
    }

    fn render_background_pixel(ppu: &Ppu) -> u16 {
        let fine_x: u8 = ppu.fine_x.into();

        if !ppu.mask.contains(Mask::BACKGROUND)
            || (fine_x < 8 && ppu.mask.contains(Mask::BACKGROUND_LEFT))
        {
            // background rendering disabled
            return 0;
        }

        let x = 15u8.wrapping_sub(fine_x);
        let mut p: u16 = ppu.bg_shift.high.nth(x) << 1 | ppu.bg_shift.high.nth(x);
        if 0 < p {
            let a: u16 = (ppu.at_shift.high.nth(x) << 1 | ppu.at_shift.high.nth(x)).into();
            p |= a << 2;
        }
        p
    }
}

impl Ppu {
    fn background_shift(&mut self) {
        self.bg_shift.low <<= 1;
        self.bg_shift.high <<= 1;
        self.at_shift.low = (self.at_shift.low << 1) | if self.at_shift.low_latch { 1 } else { 0 };
        self.at_shift.high =
            (self.at_shift.high << 1) | if self.at_shift.high_latch { 1 } else { 0 };
    }

    fn background_reload_shift(&mut self) {
        self.bg_shift.low = (self.bg_shift.low & 0xFF00) | self.bg.low;
        self.bg_shift.high = (self.bg_shift.high & 0xFF00) | self.bg.high;
        self.at_shift.low_latch = self.at_latch.nth(0) == 1;
        self.at_shift.high_latch = self.at_latch.nth(1) == 1;
    }

    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#Coarse_X_increment
    fn incr_coarse_x(&mut self) {
        if self.v.coarse_x_scroll() == 31u16.into() {
            self.v &= !0b11111; // coarse X = 0
            self.v ^= 0x0400; // switch horizontal nametable
        } else {
            self.v += 1;
        }
    }

    fn incr_y(&mut self) {
        if self.v.fine_y_scroll() < 7.into() {
            self.v += 0x1000;
        } else {
            self.v &= !0x7000; // fine Y = 0

            let mut y: u16 = self.v.coarse_y_scroll().into();
            if y == 29 {
                y = 0;
                self.v ^= 0x0800; // switch vertical nametable
            } else if y == 31 {
                y = 0;
            } else {
                y += 1;
            }

            self.v = (self.v & !0x03E0) | (y << 5);
        }
    }

    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#At_dot_257_of_each_scanline
    fn copy_x(&mut self) {
        // v: ....F.. ...EDCBA = t: ....F.. ...EDCBA
        self.v = (self.v & !0b100_00011111) | (self.t & 0b100_00011111)
    }

    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#During_dots_280_to_304_of_the_pre-render_scanline_.28end_of_vblank.29
    fn copy_y(&mut self) {
        // v: IHGF.ED CBA..... = t: IHGF.ED CBA.....
        self.v = (self.v & !0b1111011_11100000) | (self.t & 0b1111011_11100000)
    }
}
