mod vram_address;

use crate::interrupt::Interrupt;
use crate::nes::Mirroring;
use crate::prelude::*;

use self::vram_address::VRAMAddress;

const SPRITE_COUNT: usize = 64;
const SPRITE_LIMIT: usize = 8;
const OAM_SIZE: usize = 4 * SPRITE_COUNT;

const NAME_TABLE_FIRST: Word = Word::new(0x2000u16);
const ATTR_TABLE_FIRST: Word = Word::new(0x23C0u16);
const TILE_HEIGHT: Byte = Byte::new(8);

const MAX_DOT: u16 = 340;
const MAX_LINE: u16 = 261;

pub fn step(nes: &mut Nes) -> Option<Interrupt> {
    let mut interrupt = None;

    let scanline = Scanline::from(nes.scan.line);
    match scanline {
        Scanline::Pre => {
            process_sprites(nes);
            process_background(nes, scanline);
        }
        Scanline::Visible => {
            process_sprites(nes);
            process_background(nes, scanline);
            render_pixel(nes);
        }
        Scanline::Post { nmi: false } => {
            if nes.scan.dot == 0 {
                // new frame
            }
        }
        Scanline::Post { nmi: true } => {
            if nes.scan.dot == 1 {
                // Begin VBLANK
                nes.ppu.status.insert(Status::VBLANK);
                if nes.ppu.controller.contains(Controller::NMI) {
                    interrupt = Some(Interrupt::NMI);
                }
            }
        }
    }

    if let ScanEvent::NextFrame = nes.scan.next() {
        nes.frames += 1;
    }

    interrupt
}

fn process_sprites(nes: &mut Nes) {
    match nes.scan.dot {
        1 => {
            // clear OAM
            for e in nes.ppu.secondary_oam.iter_mut() {
                *e = Default::default();
            }
            nes.ppu.sprite_zero_on_line = false;
        }
        257 => {
            // eval sprites
            let sprite_size = nes.ppu.sprite_size() as u16;

            let mut iter = nes.ppu.secondary_oam.iter_mut();

            let mut n = 0;
            for i in 0..SPRITE_COUNT {
                let first = i * 4;
                let y = nes.ppu.primary_oam[first];

                if let Some(p) = iter.next() {
                    let row = nes
                        .scan
                        .line
                        .wrapping_sub(nes.ppu.primary_oam[first] as u16);
                    if row < sprite_size {
                        if n == 0 {
                            nes.ppu.sprite_zero_on_line = true;
                        }
                        *p = y;
                        *iter.next().unwrap() = nes.ppu.primary_oam[first + 1];
                        *iter.next().unwrap() = nes.ppu.primary_oam[first + 2];
                        *iter.next().unwrap() = nes.ppu.primary_oam[first + 3];
                        n += 1;
                    }
                }
            }
            nes.ppu.status.set(
                Status::SPRITE_OVERFLOW,
                SPRITE_LIMIT <= n && nes.ppu.mask.contains(Mask::RENDER_ENABLED),
            );
        }
        321 => {
            // fetch sprites
            let i = (nes.scan.dot.wrapping_sub(257)) / 8;
            let n = i.wrapping_mul(4) as usize;
            nes.ppu.sprites[i as usize] = Sprite {
                y: nes.ppu.secondary_oam[n],
                tile_index: nes.ppu.secondary_oam[n + 1],
                attr: SpriteAttribute::from_bits_truncate(nes.ppu.secondary_oam[n + 1]),
                x: nes.ppu.secondary_oam[n + 1],
            };
        }
        _ => {}
    }
}

fn process_background(nes: &mut Nes, scanline: Scanline) {
    match nes.scan.dot {
        1 => {
            nes.ppu.background_shift();
            // no shift reloading
            nes.ppu.bg_addr = NAME_TABLE_FIRST | nes.ppu.v.name_table_address_index();
            if let Scanline::Pre = scanline {
                // End VBLANK
                nes.ppu
                    .status
                    .remove(Status::VBLANK | Status::SPRITE_ZERO_HIT | Status::SPRITE_OVERFLOW);
            }
        }
        dot @ 2..=255 | dot @ 322..=336 => {
            nes.ppu.background_shift();

            // tile shift
            match dot % 8 {
                // Fetch nametable byte : step 1
                1 => {
                    nes.ppu.bg_addr = NAME_TABLE_FIRST | nes.ppu.v.name_table_address_index();
                    nes.ppu.background_reload_shift();
                }
                // Fetch nametable byte : step 2
                2 => {
                    nes.ppu.nt_latch = PpuBus::read(nes.ppu.bg_addr, nes);
                }
                // Fetch attribute table byte : step 1
                3 => {
                    nes.ppu.bg_addr = ATTR_TABLE_FIRST | nes.ppu.v.attribute_address_index();
                }
                // Fetch attribute table byte : step 2
                4 => {
                    nes.ppu.at_latch = PpuBus::read(nes.ppu.bg_addr, nes);
                    if nes.ppu.v.coarse_x_scroll().nth(0) == 1 {
                        nes.ppu.at_latch >>= 1
                    }
                    if nes.ppu.v.coarse_y_scroll().nth(0) == 1 {
                        nes.ppu.at_latch >>= 3
                    }
                }
                // Fetch tile bitmap low byte : step 1
                5 => {
                    let base: Word = if nes.ppu.controller.contains(Controller::BG_TABLE_ADDR) {
                        0x1000u16
                    } else {
                        0x0000u16
                    }
                    .into();
                    let index = nes.ppu.nt_latch * TILE_HEIGHT * 1;
                    nes.ppu.bg_addr = base + index + nes.ppu.v.fine_y_scroll();
                }
                // Fetch tile bitmap low byte : step 2
                6 => {
                    nes.ppu.bg.low = PpuBus::read(nes.ppu.bg_addr, nes).into();
                }
                // Fetch tile bitmap high byte : step 1
                7 => {
                    nes.ppu.bg_addr += <Byte as Into<Word>>::into(TILE_HEIGHT);
                }
                // Fetch tile bitmap high byte : step 2
                0 => {
                    nes.ppu.bg.high = PpuBus::read(nes.ppu.bg_addr, nes).into();
                    if nes.ppu.mask.contains(Mask::RENDER_ENABLED) {
                        nes.ppu.incr_coarse_x();
                    }
                }
                _ => panic!(),
            }
        }
        256 => {
            nes.ppu.background_shift();
            nes.ppu.bg.high = PpuBus::read(nes.ppu.bg_addr, nes).into();
            if nes.ppu.mask.contains(Mask::RENDER_ENABLED) {
                nes.ppu.incr_y();
            }
        }
        257 => {
            nes.ppu.background_reload_shift();
            if nes.ppu.mask.contains(Mask::RENDER_ENABLED) {
                nes.ppu.copy_x();
            }
        }
        279..=304 => {
            if let Scanline::Pre = scanline {
                if nes.ppu.mask.contains(Mask::RENDER_ENABLED) {
                    nes.ppu.copy_y();
                }
            }
        }
        320 => {
            // no shift reloading
            nes.ppu.bg_addr = NAME_TABLE_FIRST | nes.ppu.v.name_table_address_index();
        }
        // Unused name table fetches
        337 | 339 => {
            nes.ppu.bg_addr = NAME_TABLE_FIRST | nes.ppu.v.name_table_address_index();
        }
        338 | 340 => {
            nes.ppu.nt_latch = PpuBus::read(nes.ppu.bg_addr, nes);
        }
        341 => {
            if nes.ppu.mask.contains(Mask::RENDER_ENABLED) && nes.frames % 2 == 0 {
                // Skip 0 cycle on visible frame
                nes.scan.skip();
            }
        }
        _ => {}
    }
}

fn render_pixel(nes: &mut Nes) {
    let bg_addr = render_background_pixel(nes);

    let x = nes.scan.dot - 2;
    let (sprite_addr, attr) = render_sprite(nes, x as i32, bg_addr);

    let addr = match (0 < bg_addr, 0 < sprite_addr) {
        (false, false) => 0x3F00,
        (false, true) => 0x3F00 + sprite_addr,
        (true, false) => 0x3F00 + bg_addr,
        (true, true) => {
            if attr.contains(SpriteAttribute::BEHIND_BACKGROUND) {
                bg_addr
            } else {
                sprite_addr
            }
        }
    };

    let _pixel = PpuBus::read(addr.into(), nes);
    // render pixel
}

fn render_background_pixel(nes: &Nes) -> u16 {
    let fine_x: u8 = nes.ppu.fine_x.into();

    if !nes.ppu.mask.is_enable_background(fine_x) {
        return 0;
    }

    let x = 15u8.wrapping_sub(fine_x);
    let mut p: u16 = nes.ppu.bg_shift.high.nth(x) << 1 | nes.ppu.bg_shift.high.nth(x);
    if 0 < p {
        let a: u16 = (nes.ppu.at_shift.high.nth(x) << 1 | nes.ppu.at_shift.high.nth(x)).into();
        p |= a << 2;
    }
    p
}

fn render_sprite(nes: &mut Nes, x: i32, bg_addr: u16) -> (u16, SpriteAttribute) {
    if !nes.ppu.mask.is_enable_sprite(nes.ppu.fine_x.into()) {
        return (0, Default::default());
    }

    let y = nes.scan.line;
    for (i, sprite) in nes.ppu.sprites.clone().iter().enumerate() {
        if !sprite.valid() {
            break;
        }
        if (sprite.x as i32) < x - 7 && x < sprite.x as i32 {
            continue;
        }
        let mut row = sprite.row(y, nes.ppu.sprite_size());
        let col = sprite.col(x as u16);
        let mut tile_idx = sprite.tile_index as u16;

        let base = if nes.ppu.controller.sprite_8x16_pixels() {
            tile_idx &= 0xFE;
            if 7 < row {
                tile_idx += 1;
                row -= 8;
            }
            tile_idx & 1
        } else {
            nes.ppu.controller.base_sprite_table_addr()
        };

        let tile_addr = base + tile_idx * 16 + row;
        let low = PpuBus::read(tile_addr.into(), nes);
        let high = PpuBus::read((tile_addr + 8).into(), nes);

        let pixel = low.nth(col) + (high.nth(col) << 1);
        if pixel == 0 {
            // transparent
            continue;
        }

        if i == 0
            && nes.ppu.sprite_zero_on_line
            && !nes.ppu.status.contains(Status::SPRITE_ZERO_HIT)
            && sprite.x != 0xFF
            && x < 0xFF
            && 0 < bg_addr
        {
            nes.ppu.status.insert(Status::SPRITE_ZERO_HIT);
        }
        return (pixel.into(), sprite.attr);
    }
    (0, Default::default())
}

pub struct Ppu {
    // PPUCTRL
    controller: Controller,
    // PPUMASK
    mask: Mask,
    // PPUSTATUS
    status: Status,
    // OAMADDR
    oam_address: usize,
    // OAMDATA
    primary_oam: [u8; OAM_SIZE],
    secondary_oam: [u8; 32],
    // PPUSCROLL
    fine_x: Byte, // Fine X scroll
    // PPUADDR
    v: VRAMAddress, // current VRAM address
    t: VRAMAddress, // temporary VRAM address
    // PPUDATA
    data: Byte,

    write_toggle: bool,
    // http://wiki.nesdev.com/w/index.php/PPU_registers#Ports
    internal_data_bus: u8,

    name_table: [Byte; 0x1000],
    pallete_ram_idx: [Byte; 0x0020],

    // Background
    bg: BackgroundPattern,
    bg_addr: Word,
    nt_latch: Byte,
    at_latch: Byte,
    bg_shift: BackgroundPattern,
    at_shift: BackgroundPatternAttr,

    // Sprites
    sprites: [Sprite; SPRITE_LIMIT],
    sprite_zero_on_line: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            controller: Default::default(),
            mask: Default::default(),
            status: Default::default(),
            oam_address: Default::default(),
            primary_oam: [Default::default(); OAM_SIZE],
            secondary_oam: [Default::default(); 32],
            fine_x: Default::default(),
            v: Default::default(),
            t: Default::default(),
            data: Default::default(),
            write_toggle: false,
            internal_data_bus: 0,
            name_table: [Default::default(); 0x1000],
            pallete_ram_idx: [Default::default(); 0x0020],
            bg: Default::default(),
            bg_addr: Default::default(),
            nt_latch: Default::default(),
            at_latch: Default::default(),
            bg_shift: Default::default(),
            at_shift: Default::default(),
            sprites: [Default::default(); SPRITE_LIMIT],
            sprite_zero_on_line: false,
        }
    }

    pub fn reset(&mut self) {
        self.controller = Default::default();
        self.mask = Default::default();
        self.status = Default::default();
        self.data = 0x00.into();

        self.name_table = [Default::default(); 0x1000];
        self.pallete_ram_idx = [Default::default(); 0x0020];
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
        self.v += self.controller.vram_increment()
    }

    fn sprite_size(&self) -> i8 {
        if self.controller.contains(Controller::SPRITE_SIZE) {
            16
        } else {
            8
        }
    }
}

impl Ppu {
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
                nes.ppu.data = PpuBus::read(nes.ppu.v.into(), nes);
                data
            } else {
                PpuBus::read(nes.ppu.v.into(), nes)
            };
            nes.ppu.incr_v();
            result
        }
        _ => Default::default(),
    };

    nes.ppu.internal_data_bus = result.into();
    result
}

pub fn write_register<M: Bus, A: Into<u16>>(addr: A, value: Byte, nes: &mut Nes) {
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
            M::write(nes.ppu.v.into(), value, nes);
            nes.ppu.incr_v();
        }
        _ => {}
    }
}

pub enum PpuBus {}

impl Bus for PpuBus {
    fn read(addr: Word, nes: &mut Nes) -> Byte {
        let a: u16 = addr.into();
        match a {
            0x0000..=0x1FFF => unimplemented!("mapper"),
            0x2000..=0x2FFF => nes.ppu.name_table[to_name_table_addr(a, &nes.mirroring)],
            0x3000..=0x3EFF => {
                nes.ppu.name_table[to_name_table_addr(addr - 0x1000u16, &nes.mirroring)]
            }
            0x3F00..=0x3FFF => nes.ppu.pallete_ram_idx[to_pallete_addr(a)],
            _ => Default::default(),
        }
    }

    fn write(addr: Word, value: Byte, nes: &mut Nes) {
        let a: u16 = addr.into();
        match a {
            0x0000..=0x1FFF => unimplemented!("mapper"),
            0x2000..=0x2FFF => {
                nes.ppu.name_table[to_name_table_addr(a, &nes.mirroring)] = value.into();
            }
            0x3000..=0x3EFF => {
                nes.ppu.name_table[to_name_table_addr(addr - 0x1000u16, &nes.mirroring)] =
                    value.into();
            }
            0x3F00..=0x3FFF => {
                nes.ppu.pallete_ram_idx[to_pallete_addr(a)] = value.into();
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
    fn name_table_select(&self) -> Word {
        (self.bits() & 0b11).into()
    }

    fn base_sprite_table_addr(&self) -> u16 {
        if self.contains(Self::SPRITE_TABLE_ADDR) {
            0x1000
        } else {
            0x0000
        }
    }

    fn sprite_8x16_pixels(&self) -> bool {
        self.contains(Self::SPRITE_SIZE)
    }

    fn vram_increment(&self) -> u16 {
        if self.contains(Self::VRAM_ADDR_INCR) {
            32u16
        } else {
            1u16
        }
    }
}

bitflags! {
    #[derive(Default)]
    struct Mask: u8 {
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

impl Mask {
    fn is_enable_background(self, x: u8) -> bool {
        self.contains(Self::BACKGROUND) && !(x < 8 && !self.contains(Self::BACKGROUND_LEFT))
    }
    fn is_enable_sprite(self, x: u8) -> bool {
        self.contains(Self::SPRITE) && !(x < 8 && !self.contains(Self::SPRITE_LEFT))
    }
}

bitflags! {
    #[derive(Default)]
    struct Status: u8 {
        // In vblank?
        const VBLANK = 1 << 7;
        // Sprite 0 Hit
        const SPRITE_ZERO_HIT = 1 << 6;
        // Sprite overflow
        const SPRITE_OVERFLOW = 1 << 5;
    }
}

enum Scanline {
    Visible,
    Post { nmi: bool },
    Pre,
}

impl Scanline {
    fn from(line: u16) -> Scanline {
        match line {
            0..=239 => Self::Visible,
            240 | 242..=260 => Self::Post { nmi: false },
            241 => Self::Post { nmi: true },
            261 => Self::Pre,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Scan {
    dot: u16,
    line: u16,
}

impl Scan {
    fn clear(&mut self) {
        self.dot = 0;
        self.line = 0;
    }

    fn skip(&mut self) {
        self.dot += 1;
    }

    fn next(&mut self) -> ScanEvent {
        self.dot = self.dot.wrapping_add(1);
        if MAX_DOT <= self.dot {
            self.dot %= MAX_DOT;

            self.line += 1;
            if MAX_LINE < self.line {
                self.line = 0;
                ScanEvent::NextFrame
            } else {
                ScanEvent::NextLine
            }
        } else {
            ScanEvent::NextDot
        }
    }
}

enum ScanEvent {
    NextDot,
    NextLine,
    NextFrame,
}

#[derive(Debug, Copy, Clone, Default)]
struct Pixel {
    color: u8,
    enabled: bool,
}

#[derive(Debug, Copy, Clone, Default)]
struct BackgroundPattern {
    low: Word,
    high: Word,
}

#[derive(Debug, Copy, Clone, Default)]
struct BackgroundPatternAttr {
    low: Byte,
    high: Byte,
    low_latch: bool,
    high_latch: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
struct Sprite {
    // Y position of top
    y: u8,
    // Tile index number
    tile_index: u8,
    // Attributes
    attr: SpriteAttribute,
    // X position of left
    x: u8,
}

impl Sprite {
    pub fn valid(&self) -> bool {
        !(self.x == 0xFF && self.y == 0xFF && self.tile_index == 0xFF && self.attr.bits() == 0xFF)
    }

    pub fn row(&self, line: u16, sprite_height: i8) -> u16 {
        let row = (line as u16).wrapping_sub(self.y as u16).wrapping_sub(1);
        if self.attr.contains(SpriteAttribute::FLIP_VERTICALLY) {
            (sprite_height as u16).wrapping_sub(1).wrapping_sub(row)
        } else {
            row
        }
    }

    pub fn col(&self, x: u16) -> u8 {
        let col = 7u16.wrapping_sub(x.wrapping_sub(self.x as u16));
        if self.attr.contains(SpriteAttribute::FLIP_HORIZONTALLY) {
            8u16.wrapping_sub(1).wrapping_sub(col) as u8
        } else {
            col as u8
        }
    }
}

bitflags! {
    #[derive(Default)]
    struct SpriteAttribute: u8 {
        const FLIP_VERTICALLY = 1 << 7;
        const FLIP_HORIZONTALLY = 1 << 6;
        // Priority
        const BEHIND_BACKGROUND = 1 << 5;
    }
}
