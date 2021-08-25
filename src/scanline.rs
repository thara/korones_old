const MAX_DOT: u16 = 340;
const MAX_LINE: u16 = 261;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct Scan {
    pub(crate) dot: u16,
    pub(crate) line: u16,

    pub(crate) scanline: Scanline,

    pub(crate) frames: u64,
}

impl Scan {
    pub(crate) fn new() -> Self {
        Self {
            dot: 0,
            line: 0,
            scanline: Scanline::Visible,
            frames: 0,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.dot = 0;
        self.line = 0;
    }

    pub(crate) fn skip(&mut self) {
        self.dot += 1;
    }

    pub(crate) fn next(&mut self) {
        self.dot = self.dot.wrapping_add(1);
        if MAX_DOT <= self.dot {
            self.dot %= MAX_DOT;

            self.line += 1;
            self.scanline = Scanline::from(self.line);

            if MAX_LINE < self.line {
                self.line = 0;
                self.frames = self.frames.wrapping_add(1);
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum Scanline {
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
