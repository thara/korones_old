use std::ops;

use crate::data_unit::*;
use crate::interrupt::*;
use crate::nes::*;
use crate::rom::Mirroring;

pub const MAX_DOT: u16 = 340;
pub const MAX_LINE: i16 = 261;

const SPRITE_LIMIT: usize = 8;
const SPRITE_COUNT: usize = 64;
const OAM_SIZE: usize = 4 * SPRITE_COUNT;

const TILE_HEIGHT: Byte = Byte::new(8);

#[derive(Default)]
pub(crate) struct Ppu {
    // PPUCTRL
    ctrl: Controller,
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
    bg: Pattern,
    nt_latch: Byte,
    at_latch: Byte,
    bg_shift: Pattern,
    at_shift: PatternAttr,

    // Sprites
    sprites: [Spr; SPRITE_LIMIT],
    sprite_zero_on_line: bool,

    pub mirroring: Mirroring,

    scan: Scan,
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
        const SPR_TABLE_ADDR = 1 << 3;
        // VRAM address increment
        const VRAM_ADDR_INCR = 1 << 2;
    }
}

impl Controller {
    fn name_table_select(&self) -> Word {
        (self.bits() & 0b11).into()
    }
    fn bg_table(&self) -> Word {
        if self.contains(Controller::BG_TABLE_ADDR) {
            0x1000.into()
        } else {
            0x0000.into()
        }
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
        const BG = 1 << 3;
        // Show sprite in leftmost 8 pixels
        const SPRITE_LEFT = 1 << 2;
        // Show background in leftmost 8 pixels
        const BG_LEFT = 1 << 1;
        // Greyscale
        #[allow(dead_code)]
        const GREYSCALE = 1;

        const RENDER_ENABLED = Self::SPRITE.bits | Self::BG.bits;
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub(crate) struct Scan {
    dot: u16,
    line: i16,
}

impl Scan {
    pub(crate) fn clear(&mut self) {
        self.dot = 0;
        self.line = 0;
    }

    fn skip(&mut self) {
        self.dot += 1;
    }

    fn next(&mut self) -> bool {
        self.dot = self.dot.wrapping_add(1);
        if MAX_DOT <= self.dot {
            self.dot %= MAX_DOT;

            self.line += 1;

            if MAX_LINE < self.line {
                self.line = 0;
                return true;
            }
        }
        false
    }
}

pub(crate) fn step(nes: &mut Nes, scan: &mut Scan, frames: u64) -> u64 {
    // bg
    let render_enabled = nes.ppu.mask.contains(Mask::RENDER_ENABLED);
    let dot = scan.dot;
    let line = scan.line;
    let v = nes.ppu.v;

    match (dot, line) {
        (1..=256 | 322..=336, 261 | 0..=239) => {
            nes.ppu.bg_shift();
            match dot % 8 {
                1 => {
                    // Fetch nametable byte
                    nes.ppu.nt_latch = nes.read_ppu(v.tile_addr());
                }
                3 => {
                    // Fetch attribute table byte
                    nes.ppu.at_latch = nes.read_ppu(v.attr_addr());
                }
                5 => {
                    // Fetch tile bitmap low byte
                    let base = nes.ppu.ctrl.bg_table();
                    let index = nes.ppu.nt_latch * TILE_HEIGHT * 1;
                    let addr = base + index + v.fine_y_scroll();
                    nes.ppu.bg.low = nes.read_ppu(addr).into();
                }
                7 => {
                    // Fetch tile bitmap high byte
                    let base = nes.ppu.ctrl.bg_table();
                    let index = nes.ppu.nt_latch * TILE_HEIGHT * 1;
                    let addr = base + index + v.fine_y_scroll();
                    nes.ppu.bg.high = nes.read_ppu(addr + TILE_HEIGHT).into();
                }
                _ => {}
            }
        }
        (337 | 339, 261 | 0..=239) => {
            // Unused name table fetches
            nes.ppu.nt_latch = nes.read_ppu(v.tile_addr());
        }
        _ => {}
    }
    // scroll
    if render_enabled {
        let mut v = nes.ppu.v;
        let mut t = nes.ppu.t;
        match (dot, line) {
            (256, 261 | 0..=239) => {
                // https://wiki.nesdev.com/w/index.php?title=PPU_scrolling#Y_increment
                if (v & 0x07000) != 7.into() {
                    // fine Y < 7
                    v += 0x1000;
                } else {
                    v &= !0x7000; // fine Y = 0
                    let mut y: u16 = v.coarse_y_scroll().into();
                    if y == 29 {
                        y = 0;
                        v ^= 0x0800; // switch vertical nametable
                    } else if y == 31 {
                        y = 0;
                    } else {
                        y += 1;
                    }
                    nes.ppu.v = (v & !0x03E0) | (y << 5);
                }
            }
            (257, 261 | 0..=239) => {
                // http://wiki.nesdev.com/w/index.php/PPU_scrolling#At_dot_257_of_each_scanline
                // v: ....F.. ...EDCBA = t: ....F.. ...EDCBA
                nes.ppu.v = (v & !0b100_00011111u16) | (t & 0b100_00011111u16);
            }
            (280..=304, 261) => {
                // http://wiki.nesdev.com/w/index.php/PPU_scrolling#During_dots_280_to_304_of_the_pre-render_scanline_.28end_of_vblank.29
                // v: IHGF.ED CBA..... = t: IHGF.ED CBA.....
                nes.ppu.v = (v & !0b1111011_11100000) | (t & 0b1111011_11100000);
            }
            (9..=256, 261 | 0..=239) => {
                nes.ppu.bg_reload_shift();
            }
            (1..=256 | 322..=336, 261 | 0..=239) if dot % 8 == 7 => {
                // http://wiki.nesdev.com/w/index.php/PPU_scrolling#Coarse_X_increment
                if v.coarse_x_scroll() == 31u16.into() {
                    v = v & !0b11111; // coarse X = 0
                    v ^= 0x0400; // switch horizontal nametable
                } else {
                    v += 1;
                }
                nes.ppu.v = v;
            }
            (1..=256 | 322..=336, 261 | 0..=239) if dot % 8 == 3 => {
                if v.coarse_x_scroll().nth(0) == 1 {
                    nes.ppu.at_latch >>= 1
                }
                if v.coarse_y_scroll().nth(0) == 1 {
                    nes.ppu.at_latch >>= 3
                }
            }
            _ => {}
        }
    }

    // sprites
    match (dot, line) {
        // Pre-render/Visible scanline
        (0, 261 | 0..=239) => {
            nes.oam.clear();
            nes.ppu.sprite_zero_on_line = false;

            nes.oam.eval_sprites(&scan, &mut nes.ppu);
        }
        (257..=320, 261 | 0..=239) => {
            let (i, spr) = nes.oam.fetch_sprite(scan);
            nes.ppu.sprites[i] = spr;
        }
        _ => {}
    }

    // visible
    if let 0..=239 = line {
        let bg_addr = get_bg_pixel(nes);

        let x = dot.wrapping_sub(2);
        let (spr_addr, attr) = get_sprite_pixel(nes, x as i32, bg_addr, &scan);

        let addr = match (0 < bg_addr, 0 < spr_addr) {
            (false, false) => 0x3F00,
            (false, true) => 0x3F00 + spr_addr,
            (true, false) => 0x3F00 + bg_addr,
            (true, true) => {
                if attr.contains(SprAttr::BEHIND_BACKGROUND) {
                    bg_addr
                } else {
                    spr_addr
                }
            }
        };

        let pixel = nes.read_ppu(addr);
        nes.write_buffer(dot.into(), line as usize, pixel);
    }

    match (dot, line) {
        (_, 261) => {
            if render_enabled && frames % 2 == 0 {
                // Skip 0 cycle on visible frame
                scan.skip();
            }
        }
        (1, 241) => {
            // begin VBLANK
            nes.ppu.status.insert(Status::VBLANK);
            if nes.ppu.ctrl.contains(Controller::NMI) {
                nes.interrupt.insert(Interrupt::NMI);
            }
            nes.swap_buffers();
        }
        _ => {}
    }

    if scan.next() {
        frames + 1
    } else {
        frames
    }
}

fn get_bg_pixel(nes: &Nes) -> u16 {
    let ppu = &nes.ppu;
    let bg = &ppu.bg_shift;
    let at = &ppu.at_shift;

    let fine_x: u8 = ppu.fine_x.into();
    let mask = nes.ppu.mask;

    if !mask.contains(Mask::BG) || (fine_x < 8 && mask.contains(Mask::BG_LEFT)) {
        return 0; // background rendering disabled
    }
    let x = 15u8.wrapping_sub(fine_x);
    let p: u16 = bg.high.nth(x) << 1 | bg.low.nth(x);
    if 0 < p {
        let a: u16 = (at.high.nth(x) << 1 | at.low.nth(x)).into();
        p | (a << 2)
    } else {
        p
    }
}

fn get_sprite_pixel(nes: &mut Nes, x: i32, bg_addr: u16, scan: &Scan) -> (u16, SprAttr) {
    let fine_x: u8 = nes.ppu.fine_x.into();
    let mask = nes.ppu.mask;

    if !mask.contains(Mask::SPRITE) || (fine_x < 8 && mask.contains(Mask::SPRITE_LEFT)) {
        return (0, Default::default());
    }

    let y = scan.line;
    for (i, spr) in nes.ppu.sprites.clone().iter().enumerate() {
        if !spr.valid() {
            break;
        }
        if (spr.x as i32) < x - 7 && x < spr.x as i32 {
            continue;
        }
        let mut row = spr.row(y, nes.ppu.sprite_size());
        let col = spr.col(x as u16);
        let mut tile_idx = spr.tile_index as u16;

        let base = if nes.ppu.ctrl.contains(Controller::SPRITE_SIZE) {
            // 8x16 pixels
            tile_idx &= 0xFE;
            if 7 < row {
                tile_idx += 1;
                row -= 8;
            }
            tile_idx & 1
        } else if nes.ppu.ctrl.contains(Controller::SPR_TABLE_ADDR) {
            0x1000
        } else {
            0x0000
        };

        let tile_addr = base + tile_idx * 16 + row;
        let low = nes.read_ppu(tile_addr);
        let high = nes.read_ppu(tile_addr + 8);

        let pixel = low.nth(col) + (high.nth(col) << 1);
        if pixel == 0 {
            // transparent
            continue;
        }

        if i == 0
            && nes.ppu.sprite_zero_on_line
            && !nes.ppu.status.contains(Status::SPRITE_ZERO_HIT)
            && spr.x != 0xFF
            && x < 0xFF
            && 0 < bg_addr
        {
            nes.ppu.status.insert(Status::SPRITE_ZERO_HIT);
        }
        return (pixel.into(), spr.attr);
    }
    (0, Default::default())
}

// PPU memory map
impl Nes {
    fn read_ppu(&mut self, addr: impl Into<Word>) -> Byte {
        let addr = addr.into();
        let a: u16 = addr.into();
        match a {
            0x0000..=0x1FFF => self.mapper.read(addr),
            0x2000..=0x2FFF => self.name_table[to_name_table_addr(a, &self.ppu.mirroring)],
            0x3000..=0x3EFF => {
                self.name_table[to_name_table_addr(addr - 0x1000u16, &self.ppu.mirroring)]
            }
            0x3F00..=0x3FFF => self.pallete_ram_idx[to_pallete_addr(a)],
            _ => Default::default(),
        }
    }

    fn write_ppu(&mut self, addr: impl Into<Word>, value: impl Into<Byte>) {
        let addr = addr.into();
        let a: u16 = addr.into();
        match a {
            0x0000..=0x1FFF => unimplemented!("mapper"),
            0x2000..=0x2FFF => {
                self.name_table[to_name_table_addr(a, &self.ppu.mirroring)] = value.into();
            }
            0x3000..=0x3EFF => {
                self.name_table[to_name_table_addr(addr - 0x1000u16, &self.ppu.mirroring)] =
                    value.into();
            }
            0x3F00..=0x3FFF => {
                self.pallete_ram_idx[to_pallete_addr(a)] = value.into();
            }
            _ => {}
        }
    }
}

fn to_name_table_addr(base: impl Into<u16>, mirroring: &Mirroring) -> usize {
    let base = base.into();
    match mirroring {
        Mirroring::Vertical => base % 0x0800,
        Mirroring::Horizontal => {
            if 0x2000 <= base {
                0x0800u16.wrapping_sub(base) % 0x0400
            } else {
                base % 0x0400
            }
        }
    }
    .into()
}

fn to_pallete_addr(base: u16) -> usize {
    // http://wiki.nesdev.com/w/index.php/PPU_palettes#Memory_Map
    let addr = base % 32;
    if addr % 4 == 0 { addr | 0x10 } else { addr }.into()
}

// register access from bus
impl Nes {
    pub(crate) fn read_ppu_register(&mut self, addr: impl Into<u16>) -> Byte {
        let result = match addr.into() {
            0x2002u16 => {
                let result = self.ppu.read_status() | (self.ppu.internal_data_bus & 0b11111);
                if self.ppu.scan.line == 241 && self.ppu.scan.dot < 2 {
                    result & !0x80
                } else {
                    result
                }
            }
            0x2004u16 => {
                // https://wiki.selfdev.com/w/index.php/PPU_sprite_evaluation
                if self.ppu.scan.line < 240 && 1 <= self.ppu.scan.dot && self.ppu.scan.dot <= 64 {
                    // during sprite evaluation
                    0xFF
                } else {
                    self.oam.primary[self.ppu.oam_address]
                }
                .into()
            }
            0x2007u16 => {
                let v: u16 = self.ppu.v.into();
                let result = if v <= 0x3EFFu16 {
                    let data = self.ppu.data;
                    self.ppu.data = self.read_ppu(self.ppu.v);
                    data
                } else {
                    self.read_ppu(self.ppu.v)
                };
                self.ppu.incr_v();
                result
            }
            _ => Default::default(),
        };

        self.ppu.internal_data_bus = result.into();
        result
    }

    pub(crate) fn write_ppu_register(&mut self, addr: impl Into<Word>, value: Byte) {
        let addr = addr.into();
        let addr: u16 = addr.into();
        match addr.into() {
            0x2000u16 => self.ppu.write_controller(value.into()),
            0x2001 => self.ppu.mask = Mask::from_bits_truncate(value.into()),
            0x2003 => {
                let addr: u16 = value.into();
                self.ppu.oam_address = addr.into();
            }
            0x2004 => {
                self.oam.primary[self.ppu.oam_address] = value.into();
                self.ppu.oam_address = self.ppu.oam_address.wrapping_add(1);
            }
            0x2005 => self.ppu.write_scroll(value),
            0x2006 => self.ppu.write_vram_address(value),
            0x2007 => {
                self.write_ppu(self.ppu.v, value);
                self.ppu.incr_v();
            }
            _ => {}
        }
    }
}

// register access
impl Ppu {
    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#.242000_write
    fn write_controller(&mut self, value: u8) {
        self.ctrl = Controller::from_bits_truncate(value);
        // t: ...BA.. ........ = d: ......BA
        self.t = (self.t & !0b0001100_00000000) | (self.ctrl.name_table_select() << 10)
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
        self.v += if self.ctrl.contains(Controller::VRAM_ADDR_INCR) {
            32u16
        } else {
            1u16
        };
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct Pattern {
    pub(super) low: Word,
    pub(super) high: Word,
}

impl Pattern {
    fn nth(&self, x: u8) -> u16 {
        self.high.nth(x) << 1 | self.low.nth(x)
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct PatternAttr {
    low: Byte,
    high: Byte,
    low_latch: bool,
    high_latch: bool,
}

impl Ppu {
    fn bg_shift(&mut self) {
        self.bg_shift.low <<= 1;
        self.bg_shift.high <<= 1;
        self.at_shift.low = (self.at_shift.low << 1) | if self.at_shift.low_latch { 1 } else { 0 };
        self.at_shift.high =
            (self.at_shift.high << 1) | if self.at_shift.high_latch { 1 } else { 0 };
    }

    fn bg_reload_shift(&mut self) {
        self.bg_shift.low = (self.bg_shift.low & 0xFF00) | self.bg.low;
        self.bg_shift.high = (self.bg_shift.high & 0xFF00) | self.bg.high;
        self.at_shift.low_latch = self.at_latch.nth(0) == 1;
        self.at_shift.high_latch = self.at_latch.nth(1) == 1;
    }
}

// sprites
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
struct Spr {
    // Y position of top
    y: u8,
    // Tile index number
    tile_index: u8,
    // Attributes
    attr: SprAttr,
    // X position of left
    x: u8,
}

bitflags! {
    #[derive(Default)]
    struct SprAttr: u8 {
        const FLIP_VERTICALLY = 1 << 7;
        const FLIP_HORIZONTALLY = 1 << 6;
        // Priority
        const BEHIND_BACKGROUND = 1 << 5;
    }
}

impl Spr {
    fn valid(&self) -> bool {
        !(self.x == 0xFF && self.y == 0xFF && self.tile_index == 0xFF && self.attr.bits() == 0xFF)
    }

    fn row(&self, line: i16, sprite_height: i8) -> u16 {
        let row = (line as u16).wrapping_sub(self.y as u16).wrapping_sub(1);
        if self.attr.contains(SprAttr::FLIP_VERTICALLY) {
            (sprite_height as u16).wrapping_sub(1).wrapping_sub(row)
        } else {
            row
        }
    }

    fn col(&self, x: u16) -> u8 {
        let col = 7u16.wrapping_sub(x.wrapping_sub(self.x as u16));
        if self.attr.contains(SprAttr::FLIP_HORIZONTALLY) {
            8u16.wrapping_sub(1).wrapping_sub(col) as u8
        } else {
            col as u8
        }
    }
}

pub struct Oam {
    // OAMDATA
    primary: [u8; OAM_SIZE],
    secondary: [u8; 32],
}

impl Oam {
    pub fn new() -> Self {
        Self {
            primary: [Default::default(); OAM_SIZE],
            secondary: [Default::default(); 32],
        }
    }

    fn clear(&mut self) {
        for e in self.secondary.iter_mut() {
            *e = Default::default();
        }
    }

    fn eval_sprites(&mut self, scan: &Scan, ppu: &mut Ppu) {
        let sprite_size = ppu.sprite_size() as i16;
        let mut iter = self.secondary.iter_mut();

        let mut n = 0;
        for i in 0..SPRITE_COUNT {
            let first = i * 4;
            let y = self.primary[first];

            if let Some(p) = iter.next() {
                let row = scan.line.wrapping_sub(self.primary[first] as i16);
                if row < sprite_size {
                    if n == 0 {
                        ppu.sprite_zero_on_line = true;
                    }
                    *p = y;
                    *iter.next().unwrap() = self.primary[first + 1];
                    *iter.next().unwrap() = self.primary[first + 2];
                    *iter.next().unwrap() = self.primary[first + 3];
                    n += 1;
                }
            }
        }
    }

    fn fetch_sprite(&self, scan: &Scan) -> (usize, Spr) {
        let i = (scan.dot.wrapping_sub(257)) / 8;
        let n = i.wrapping_mul(4) as usize;
        (
            i as usize,
            Spr {
                y: self.secondary[n],
                tile_index: self.secondary[n + 1],
                attr: SprAttr::from_bits_truncate(self.secondary[n + 1]),
                x: self.secondary[n + 1],
            },
        )
    }
}

impl Ppu {
    fn sprite_size(&self) -> i8 {
        if self.ctrl.contains(Controller::SPRITE_SIZE) {
            16
        } else {
            8
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
struct VramAddr(Word);

// Tile and attribute fetching
// https://wiki.nesdev.com/w/index.php/PPU_scrolling#Tile_and_attribute_fetching
//
// NN 1111 YYY XXX
// || |||| ||| +++-- high 3 bits of coarse X (x/4)
// || |||| +++------ high 3 bits of coarse Y (y/4)
// || ++++---------- attribute offset (960 bytes)
// ++--------------- nametable select
impl VramAddr {
    fn tile_addr(&self) -> u16 {
        let v: u16 = self.0.into();
        0x2000u16 | (v & 0xFFFu16)
    }

    fn attr_addr(&self) -> u16 {
        let v: u16 = self.0.into();
        0x23C0u16 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07)
    }
}

// https://wiki.nesdev.com/w/index.php/PPU_scrolling#PPU_internal_registers
//
// yyy NN YYYYY XXXXX
// ||| || ||||| +++++-- coarse X scroll
// ||| || +++++-------- coarse Y scroll
// ||| ++-------------- nametable select
// +++----------------- fine Y scroll
impl VramAddr {
    fn coarse_x_scroll(&self) -> Word {
        self.0 & 0b11111
    }

    fn coarse_y_scroll(&self) -> Word {
        self.0 & 0b11_11100000 >> 5
    }

    fn fine_y_scroll(&self) -> Byte {
        ((self.0 & 0x7000) >> 12).byte()
    }
}

impl From<u16> for VramAddr {
    fn from(value: u16) -> Self {
        Self(Word::from(value))
    }
}

impl From<Word> for VramAddr {
    fn from(value: Word) -> Self {
        Self(value)
    }
}

impl From<VramAddr> for Word {
    fn from(value: VramAddr) -> Self {
        value.0
    }
}

impl From<VramAddr> for u16 {
    fn from(value: VramAddr) -> Self {
        value.0.into()
    }
}

impl ops::AddAssign<u16> for VramAddr {
    fn add_assign(&mut self, other: u16) {
        *self = Self(self.0 + other)
    }
}

impl ops::BitAnd<u16> for VramAddr {
    type Output = Self;

    fn bitand(self, rhs: u16) -> Self::Output {
        Self(self.0 & rhs)
    }
}

impl ops::BitAndAssign<u16> for VramAddr {
    fn bitand_assign(&mut self, rhs: u16) {
        *self = Self(self.0 & rhs)
    }
}

impl ops::BitOr for VramAddr {
    type Output = Self;

    fn bitor(self, Self(rhs): Self) -> Self::Output {
        Self(self.0 | rhs)
    }
}

impl ops::BitOr<u16> for VramAddr {
    type Output = Self;

    fn bitor(self, rhs: u16) -> Self::Output {
        Self(self.0 | rhs)
    }
}

impl ops::BitOr<Word> for VramAddr {
    type Output = Self;

    fn bitor(self, rhs: Word) -> Self::Output {
        Self(self.0 | rhs)
    }
}

impl ops::BitXorAssign<u16> for VramAddr {
    fn bitxor_assign(&mut self, rhs: u16) {
        *self = Self(self.0 ^ rhs)
    }
}
