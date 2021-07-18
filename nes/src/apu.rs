use crate::prelude::*;

#[derive(Debug, Default)]
pub struct Apu {
    sampling_rate: u32,
    frame_period: u32,

    pulse1: channel::Pulse,
    pulse2: channel::Pulse,
    triangle: channel::Triangle,
    noise: channel::Noise,
    dmc: channel::DMC,

    cycles: u64,

    frame_counter_control: Byte,
    frame_sequence_step: u8,
    frame_interrupted: bool,
}

impl Apu {
    pub fn new(sampling_rate: u32, frame_period: u32) -> Self {
        Self {
            sampling_rate,
            frame_period,
            pulse1: channel::Pulse::channel_1(),
            pulse2: channel::Pulse::channel_2(),
            triangle: channel::Triangle::new(),
            noise: channel::Noise::new(),
            dmc: channel::DMC::new(),
            ..Default::default()
        }
    }

    pub fn reset(&mut self) {
        self.write(0x4017u16, 0.into()); // frame irq enabled
        self.write(0x4015u16, 0.into()); // all channels disabled
        for addr in 0x4000u16..=0x400Fu16 {
            self.write(addr, 0.into());
        }
        for addr in 0x4010u16..=0x4013u16 {
            self.write(addr, 0.into());
        }
    }

    pub fn read_status(&mut self) -> Byte {
        let mut v: u8 = 0;
        if self.dmc.interrupted {
            v |= 0x80
        }
        if self.frame_interrupted && !self.frame_interrupt_inhibit() {
            v |= 0x40
        }
        if 0 < self.dmc.bytes_remaining_counter {
            v |= 0x20
        }
        if 0 < self.noise.length_counter {
            v |= 0x08
        }
        if 0 < self.triangle.length_counter {
            v |= 0x04
        }
        if 0 < self.pulse2.length_counter {
            v |= 0x02
        }
        if 0 < self.pulse1.length_counter {
            v |= 0x01
        }
        self.frame_interrupted = false;
        v.into()
    }

    fn frame_interrupt_inhibit(&self) -> bool {
        self.frame_counter_control.nth(6) == 1
    }

    pub fn write(&mut self, addr: impl Into<u16>, value: Byte) {
        let addr: u16 = addr.into();
        match addr {
            0x4000..=0x4003 => self.pulse1.write(addr, value),
            0x4004..=0x4007 => self.pulse2.write(addr, value),
            0x4008..=0x400B => self.triangle.write(addr, value),
            0x400C..=0x400F => self.noise.write(addr, value),
            0x4010..=0x4013 => self.dmc.write(addr, value),
            0x4015 => {
                self.pulse1.set_enabled(value.nth(0) == 1);
                self.pulse2.set_enabled(value.nth(1) == 1);
                self.triangle.set_enabled(value.nth(2) == 1);
                self.noise.set_enabled(value.nth(3) == 1);
                self.dmc.set_enabled(value.nth(4) == 1);
            }
            0x4017 => self.frame_counter_control = value,
            _ => {}
        }
    }
}

mod channel {
    use crate::prelude::*;

    #[rustfmt::skip]
    static LENGTH_TABLE: [u32; 32] = [
        10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14,
        12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
    ];

    #[derive(Debug, Default)]
    pub(super) struct Pulse {
        volume: Byte,
        sweep: Byte,
        low: Byte,
        high: Byte,

        pub(super) length_counter: u32,

        enabled: bool,

        timer_counter: u16,
        timer_sequencer: i32,
        timer_period: u16,

        envelope_counter: u16,
        envelope_decay_level_counter: u8,
        envelope_start: bool,

        sweep_counter: u8,
        sweep_reload: bool,

        carry_mode: CarryMode,
    }

    #[derive(Debug)]
    pub(super) enum CarryMode {
        OnesComplement,
        TwosComplement,
    }

    impl Default for CarryMode {
        fn default() -> Self {
            CarryMode::OnesComplement
        }
    }

    impl Pulse {
        pub(super) fn channel_1() -> Self {
            Pulse {
                carry_mode: CarryMode::OnesComplement,
                ..Default::default()
            }
        }

        pub(super) fn channel_2() -> Self {
            Pulse {
                carry_mode: CarryMode::TwosComplement,
                ..Default::default()
            }
        }

        pub(super) fn write(&mut self, addr: impl Into<u16>, value: Byte) {
            match addr.into() {
                0x4000u16 => self.volume = value,
                0x4001u16 => {
                    self.sweep = value;
                    self.sweep_reload = true;
                }
                0x4002u16 => {
                    self.low = value;
                    self.timer_period = self.timer_reload();
                }
                0x4003u16 => {
                    self.high = value;
                    if self.enabled {
                        self.length_counter = LENGTH_TABLE[self.length_counter_load()];
                    }
                    self.timer_period = self.timer_reload();
                    self.timer_sequencer = 0;
                    self.envelope_start = true;
                }
                _ => {}
            }
        }

        pub(super) fn set_enabled(&mut self, v: bool) {
            self.enabled = v;
            if !v {
                self.length_counter = 0;
            }
        }

        fn timer_reload(&self) -> u16 {
            (self.low.u8() as u16) | ((self.high.u8() as u16) << 8)
        }

        fn length_counter_load(&self) -> usize {
            ((self.high & 0b11111000) >> 3).into()
        }
    }

    #[derive(Debug, Default)]
    pub(super) struct Triangle {
        linear_counter_setup: Byte,
        low: Byte,
        high: Byte,

        linear_counter_reload_flag: bool,

        timer_counter: u16,
        sequencer: u8,

        linear_counter: u8,
        pub(super) length_counter: u32,

        enabled: bool,
    }

    impl Triangle {
        pub(super) fn new() -> Self {
            Self {
                ..Default::default()
            }
        }

        pub(super) fn write(&mut self, addr: impl Into<u16>, value: Byte) {
            let addr: u16 = addr.into();
            match addr {
                0x4008 => self.linear_counter_setup = value,
                0x400A => self.low = value,
                0x400B => {
                    self.high = value;
                    self.linear_counter_reload_flag = true;
                    if self.enabled {
                        self.length_counter = LENGTH_TABLE[self.length_counter_load()];
                    }
                }
                _ => {}
            }
        }

        pub(super) fn set_enabled(&mut self, v: bool) {
            self.enabled = v;
            if !v {
                self.length_counter = 0;
            }
        }

        fn length_counter_load(&self) -> usize {
            ((self.high & 0b11111000) >> 3).into()
        }
    }

    #[derive(Debug, Default)]
    pub(super) struct Noise {
        envelope: Byte,
        period: Byte,

        envelope_counter: u8,
        envelope_decay_level_counter: u8,
        envelope_start: bool,

        shift_register: Byte,
        pub(super) length_counter: u32,

        timer_counter: u16,
        timer_period: u16,

        enabled: bool,
    }

    #[rustfmt::skip]
    static NOISE_TIMER_PERIOD_TABLE: [u16; 16] = [
        4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
    ];

    impl Noise {
        pub(super) fn new() -> Self {
            Self {
                shift_register: 1u8.into(),
                ..Default::default()
            }
        }

        pub(super) fn write(&mut self, addr: impl Into<u16>, value: Byte) {
            let addr: u16 = addr.into();
            match addr {
                0x400C => self.envelope = value,
                0x400E => {
                    self.period = value;
                    self.timer_period = NOISE_TIMER_PERIOD_TABLE[self.timer_entry()];
                }
                0x400F => {
                    if self.enabled {
                        self.length_counter = LENGTH_TABLE[(value.u16() & 0b11111000 >> 3) as usize]
                    }
                }
                _ => {}
            }
        }

        pub(super) fn set_enabled(&mut self, v: bool) {
            self.enabled = v;
            if !v {
                self.length_counter = 0;
            }
        }

        fn timer_entry(&self) -> usize {
            ((self.period.u8() as u16) & 0b1111) as usize
        }
    }

    #[derive(Debug, Default)]
    pub(super) struct DMC {
        flags: Byte,
        direct: Byte,
        address: Byte,
        length: Byte,

        timer_counter: u8,

        bits_remaining_counter: u8,

        enabled: bool,

        sample_buffer: u8,

        // memory reader
        address_counter: u8,
        pub(super) bytes_remaining_counter: u8,

        output_level: u8,

        silence: bool,
        sample_buffer_empty: bool,

        shift_register: Byte,

        pub(super) interrupted: bool,
    }

    impl DMC {
        pub(super) fn new() -> Self {
            Self {
                ..Default::default()
            }
        }

        pub(super) fn write(&mut self, addr: impl Into<u16>, value: Byte) {
            let addr: u16 = addr.into();
            match addr {
                0x4010 => self.flags = value,
                0x4011 => {
                    self.direct = value;
                    self.output_level = self.direct_load();
                }
                0x4012 => self.address = value,
                0x4013 => self.length = value,
                _ => {}
            }
        }

        pub(super) fn set_enabled(&mut self, v: bool) {
            self.enabled = v;
        }

        fn direct_load(&self) -> u8 {
            self.direct.u8() & 0b011111111
        }
    }
}
