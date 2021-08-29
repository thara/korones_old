use crate::data_unit::*;

pub trait Controller {
    fn write(&mut self, value: Byte);
    fn read(&mut self) -> Byte;

    fn update(&mut self, state: Byte);
}

pub(crate) struct Empty {}

impl Controller for Empty {
    fn write(&mut self, _: Byte) {}
    fn read(&mut self) -> Byte {
        Default::default()
    }
    fn update(&mut self, _: Byte) {}
}

#[derive(Debug, Default)]
pub struct StandardController {
    state: Byte,
    current: Byte,
    strobe: bool,
}

impl Controller for StandardController {
    fn write(&mut self, value: Byte) {
        self.strobe = value.nth(0) == 1;
        self.current = 1.into();
    }

    fn read(&mut self) -> Byte {
        let b: u8 = if self.strobe {
            0x40u8 & self.state.nth(Button::A.bits())
        } else {
            let input = self.state & self.current;

            self.current <<= 1;
            0x40u8 | (if 0u8 < input.u8() { 1 } else { 0 })
        };
        b.into()
    }

    fn update(&mut self, state: Byte) {
        self.state = state;
    }
}

bitflags! {
    #[derive(Default)]
    pub struct Button: u8 {
        const A = 1 << 0;
        const B = 1 << 1;
        const SELECT = 1 << 2;
        const START = 1 << 3;
        const UP = 1 << 4;
        const DOWN = 1 << 5;
        const LEFT = 1 << 6;
        const RIGHT = 1 << 7;
    }
}
