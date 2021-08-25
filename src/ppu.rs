mod bg;
mod spr;
mod vram_addr;

use bg::*;
use spr::*;
use vram_addr::*;

use crate::data_unit::*;
use crate::rom::Mirroring;

const SPRITE_LIMIT: usize = 8;
const SPRITE_COUNT: usize = 64;
const OAM_SIZE: usize = 4 * SPRITE_COUNT;

pub(crate) struct Ppu {
    // PPUCTRL
    controller: Controller,
    // PPUMASK
    mask: Mask,
    // PPUSTATUS
    status: Status,
    // OAMADDR
    oam_address: usize,

    // PPUSCROLL
    fine_x: Byte, // Fine X scroll
    // PPUADDR
    v: VramAddr, // current VRAM address
    t: VramAddr, // temporary VRAM address
    // PPUDATA
    data: Byte,

    write_toggle: bool,
    // http://wiki.nesdev.com/w/index.php/PPU_registers#Ports
    internal_data_bus: u8,

    // Background
    bg: BgPattern,
    bg_addr: Word,
    nt_latch: Byte,
    at_latch: Byte,
    bg_shift: BgPattern,
    at_shift: BgPatternAttr,

    // Sprites
    sprites: [Spr; SPRITE_LIMIT],
    sprite_zero_on_line: bool,

    pub mirroring: Mirroring,
}

bitflags! {
    #[derive(Default)]
    struct Controller: u8 {
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
