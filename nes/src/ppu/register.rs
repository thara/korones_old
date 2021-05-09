use super::Ppu;
use crate::prelude::*;

bitflags! {
    #[derive(Default)]
    pub(super) struct Controller: u8 {
        // NMI
        const NMI = 1 << 7;
        // PPU master/slave (0 = master, 1 = slave)
        #[allow(dead_code)]
        const SLAVE = 1 << 6;
        // Sprite size
        const SPRITE_SIZE = 1 << 5;
        // Background pattern table address
        const BG_TABLE_ADDR = 1 << 4;
        // Sprite pattern table address for 8x8 sprites
        const SPRITE_TABLE_ADDR = 1 << 3;
        // VRAM address increment
        const VRAM_ADDR_INCR = 1 << 2;
    }
}

impl Controller {
    pub(super) fn name_table_select(&self) -> Word {
        (self.bits() & 0b11).into()
    }
}

bitflags! {
    #[derive(Default)]
    pub(super) struct Mask: u8 {
        // Emphasize blue
        #[allow(dead_code)]
        const BLUE = 1 << 7;
        // Emphasize green
        #[allow(dead_code)]
        const GREEN = 1 << 6;
        // Emphasize red
        #[allow(dead_code)]
        const RED = 1 << 5;
        // Show sprite
        const SPRITE = 1 << 4;
        // Show background
        const BACKGROUND = 1 << 3;
        // Show sprite in leftmost 8 pixels
        const SPRITE_LEFT = 1 << 2;
        // Show background in leftmost 8 pixels
        const BACKGROUND_LEFT = 1 << 1;
        // Greyscale
        #[allow(dead_code)]
        const GREYSCALE = 1;

        const RENDER_ENABLED = Self::SPRITE.bits | Self::BACKGROUND.bits;
    }
}

bitflags! {
    #[derive(Default)]
    pub(super) struct Status: u8 {
        // In vblank?
        const VBLANK = 1 << 7;
        // Sprite 0 Hit
        const SPRITE_ZERO_HIT = 1 << 6;
        // Sprite overflow
        const SPRITE_OVERFLOW = 1 << 5;
    }
}

pub fn read_register(addr: impl Into<u16>, nes: &mut Nes) -> Byte {
    let result = match addr.into() {
        0x2002u16 => {
            let result = nes.ppu.read_status() | (nes.ppu.internal_data_bus & 0b11111);
            if nes.scan.line == 241 && nes.scan.dot < 2 {
                result & !0x80
            } else {
                result
            }
        }
        0x2004u16 => {
            // https://wiki.nesdev.com/w/index.php/PPU_sprite_evaluation
            if nes.scan.line < 240 && 1 <= nes.scan.dot && nes.scan.dot <= 64 {
                // during sprite evaluation
                0xFF
            } else {
                nes.ppu.primary_oam[nes.ppu.oam_address]
            }
            .into()
        }
        0x2007u16 => {
            let v: u16 = nes.ppu.v.into();
            let result = if v <= 0x3EFFu16 {
                let data = nes.ppu.data;
                nes.ppu.data = nes.ppu.read(nes.ppu.v.into(), &nes.mirroring);
                data
            } else {
                nes.ppu.read(nes.ppu.v.into(), &nes.mirroring)
            };
            nes.ppu.incr_v();
            result
        }
        _ => Default::default(),
    };

    nes.ppu.internal_data_bus = result.into();
    result
}

pub fn write_register(addr: impl Into<u16>, value: Byte, nes: &mut Nes) {
    match addr.into() {
        0x2000u16 => nes.ppu.write_controller(value.into()),
        0x2001 => nes.ppu.mask = Mask::from_bits_truncate(value.into()),
        0x2003 => {
            let addr: u16 = value.into();
            nes.ppu.oam_address = addr.into();
        }
        0x2004 => {
            nes.ppu.primary_oam[nes.ppu.oam_address] = value.into();
            nes.ppu.oam_address = nes.ppu.oam_address.wrapping_add(1);
        }
        0x2005 => nes.ppu.write_scroll(value),
        0x2006 => nes.ppu.write_vram_address(value),
        0x2007 => {
            nes.ppu.write(nes.ppu.v.into(), value, &nes.mirroring);
            nes.ppu.incr_v();
        }
        _ => {}
    }
}

// register access
impl Ppu {
    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#.242000_write
    fn write_controller(&mut self, value: u8) {
        self.controller = Controller::from_bits_truncate(value);
        // t: ...BA.. ........ = d: ......BA
        self.t = (self.t & !0b0001100_00000000) | (self.controller.name_table_select() << 10)
    }

    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#.242002_read
    fn read_status(&mut self) -> Byte {
        let s = self.status;
        self.status.remove(Status::VBLANK);
        self.write_toggle = false;
        s.bits().into()
    }

    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#.242005_first_write_.28w_is_0.29
    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#.242005_second_write_.28w_is_1.29
    fn write_scroll(&mut self, position: impl Into<u8>) {
        let p = position.into();
        if !self.write_toggle {
            // first write
            // t: ....... ...HGFED = d: HGFED...
            // x:              CBA = d: .....CBA
            self.t = (self.t & !0b0000000_00011111) | ((p as u16 & 0b11111000) >> 3);
            self.fine_x = Byte::from(p & 0b111);
            self.write_toggle = true;
        } else {
            // second write
            // t: CBA..HG FED..... = d: HGFEDCBA
            self.t = (self.t & !0b1110011_11100000)
                | ((p as u16 & 0b111) << 12)
                | ((p as u16 & 0b11111000) << 2);
            self.write_toggle = false
        }
    }

    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#.242006_first_write_.28w_is_0.29
    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#.242006_second_write_.28w_is_1.29
    fn write_vram_address(&mut self, addr: impl Into<u8>) {
        let d = addr.into();
        if !self.write_toggle {
            // first write
            // t: .FEDCBA ........ = d: ..FEDCBA
            // t: X...... ........ = 0
            self.t = (self.t & !0b0111111_00000000) | ((d as u16 & 0b111111) << 8);
            self.write_toggle = true
        } else {
            // second write
            // t: ....... HGFEDCBA = d: HGFEDCBA
            // v                   = t
            self.t = (self.t & !0b0000000_11111111) | d as u16;
            self.v = self.t;
            self.write_toggle = false
        }
    }

    fn incr_v(&mut self) {
        self.v += if self.controller.contains(Controller::VRAM_ADDR_INCR) {
            32u16
        } else {
            1u16
        };
    }
}
