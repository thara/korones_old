use super::*;
use crate::bus::*;
use crate::scanline::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub(super) struct Spr {
    // Y position of top
    y: u8,
    // Tile index number
    tile_index: u8,
    // Attributes
    attr: SprAttr,
    // X position of left
    x: u8,
}

impl Spr {
    fn valid(&self) -> bool {
        !(self.x == 0xFF && self.y == 0xFF && self.tile_index == 0xFF && self.attr.bits() == 0xFF)
    }

    fn row(&self, line: u16, sprite_height: i8) -> u16 {
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

bitflags! {
    #[derive(Default)]
    struct SprAttr: u8 {
        const FLIP_VERTICALLY = 1 << 7;
        const FLIP_HORIZONTALLY = 1 << 6;
        // Priority
        const BEHIND_BACKGROUND = 1 << 5;
    }
}

struct Oam {
    // OAMDATA
    primary: [u8; OAM_SIZE],
    secondary: [u8; 32],
}

trait SprProcessor: Bus {
    fn process(&mut self, ppu: &mut Ppu, oam: &mut Oam, scan: Scan) {
        match scan.dot {
            1 => {
                // clear OAM
                for e in oam.secondary.iter_mut() {
                    *e = Default::default();
                }
                ppu.sprite_zero_on_line = false;
            }
            257 => {
                // eval sprites
                let sprite_size = ppu.sprite_size() as u16;

                let mut iter = oam.secondary.iter_mut();

                let mut n = 0;
                for i in 0..SPRITE_COUNT {
                    let first = i * 4;
                    let y = oam.primary[first];

                    if let Some(p) = iter.next() {
                        let row = scan.line.wrapping_sub(oam.primary[first] as u16);
                        if row < sprite_size {
                            if n == 0 {
                                ppu.sprite_zero_on_line = true;
                            }
                            *p = y;
                            *iter.next().unwrap() = oam.primary[first + 1];
                            *iter.next().unwrap() = oam.primary[first + 2];
                            *iter.next().unwrap() = oam.primary[first + 3];
                            n += 1;
                        }
                    }
                }
                ppu.status.set(
                    Status::SPRITE_OVERFLOW,
                    SPRITE_LIMIT <= n && ppu.mask.contains(Mask::RENDER_ENABLED),
                );
            }
            257..=320 => {
                // fetch sprites
                let i = (scan.dot.wrapping_sub(257)) / 8;
                let n = i.wrapping_mul(4) as usize;
                ppu.sprites[i as usize] = Spr {
                    y: oam.secondary[n],
                    tile_index: oam.secondary[n + 1],
                    attr: SprAttr::from_bits_truncate(oam.secondary[n + 1]),
                    x: oam.secondary[n + 1],
                };
            }
            _ => {}
        }
    }

    fn render_sprite(&mut self, ppu: &mut Ppu, bg_addr: u16, scan: Scan) -> (u16, SprAttr) {
        let fine_x: u8 = ppu.fine_x.into();

        if !ppu.mask.contains(Mask::SPRITE) || (fine_x < 8 && ppu.mask.contains(Mask::SPRITE_LEFT))
        {
            return (0, Default::default());
        }

        let x = scan.dot.wrapping_sub(2) as i32;
        let y = scan.line;

        for (i, sprite) in ppu.sprites.clone().iter().enumerate() {
            if !sprite.valid() {
                break;
            }
            if (sprite.x as i32) < x - 7 && x < sprite.x as i32 {
                continue;
            }
            let mut row = sprite.row(y, ppu.sprite_size());
            let col = sprite.col(x as u16);
            let mut tile_idx = sprite.tile_index as u16;

            let base = if ppu.controller.contains(Controller::SPRITE_SIZE) {
                // 8x16 pixels
                tile_idx &= 0xFE;
                if 7 < row {
                    tile_idx += 1;
                    row -= 8;
                }
                tile_idx & 1
            } else if ppu.controller.contains(Controller::SPRITE_TABLE_ADDR) {
                0x1000
            } else {
                0x0000
            };

            let tile_addr = base + tile_idx * 16 + row;
            let low = self.read(tile_addr);
            let high = self.read(tile_addr + 8);

            let pixel = low.nth(col) + (high.nth(col) << 1);
            if pixel == 0 {
                // transparent
                continue;
            }

            if i == 0
                && ppu.sprite_zero_on_line
                && !ppu.status.contains(Status::SPRITE_ZERO_HIT)
                && sprite.x != 0xFF
                && x < 0xFF
                && 0 < bg_addr
            {
                ppu.status.insert(Status::SPRITE_ZERO_HIT);
            }
            return (pixel.into(), sprite.attr);
        }
        (0, Default::default())
    }
}

impl Ppu {
    fn sprite_size(&self) -> i8 {
        if self.controller.contains(Controller::SPRITE_SIZE) {
            16
        } else {
            8
        }
    }
}
