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

    cycles: u32,

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

pub fn step<B: Bus>(nes: &mut Nes) -> bool {
    nes.apu.cycles += 1;

    // Down sampling
    if nes.apu.cycles % nes.apu.sampling_rate == 0 {
        //TODO send audio
    }

    let cpu_stall = if nes.apu.cycles % 2 == 0 {
        nes.apu.pulse1.clock_timer();
        nes.apu.pulse2.clock_timer();
        nes.apu.noise.clock_timer();
        channel::dmc_clock_timer::<B>(nes)
    } else {
        false
    };

    nes.apu.triangle.clock_timer();

    if nes.apu.cycles % nes.apu.frame_period == 0 {
        if nes.apu.frame_counter_control.nth(7) == 0 {
            // four step
            nes.apu.pulse1.clock_envelope();
            nes.apu.pulse2.clock_envelope();
            nes.apu.triangle.clock_linear_counter();
            nes.apu.noise.clock_envelope();
            if let 1 | 3 = nes.apu.frame_sequence_step {
                nes.apu.pulse1.clock_length_counter();
                nes.apu.pulse1.clock_sweep_unit();
                nes.apu.pulse2.clock_length_counter();
                nes.apu.pulse2.clock_sweep_unit();
                nes.apu.triangle.clock_length_counter();
                nes.apu.noise.clock_length_counter();
            }
            if nes.apu.frame_sequence_step == 3 && !nes.apu.frame_interrupt_inhibit() {
                nes.apu.frame_interrupted = true;
            }
            nes.apu.frame_sequence_step = (nes.apu.frame_sequence_step + 1) % 4;
        } else {
            // five step
            if nes.apu.frame_sequence_step < 4 || nes.apu.frame_sequence_step == 5 {
                nes.apu.pulse1.clock_length_counter();
                nes.apu.pulse2.clock_length_counter();
                nes.apu.triangle.clock_linear_counter();
                nes.apu.noise.clock_envelope();
            }

            if nes.apu.frame_sequence_step == 1 || nes.apu.frame_sequence_step == 4 {
                nes.apu.pulse1.clock_length_counter();
                nes.apu.pulse1.clock_sweep_unit();
                nes.apu.pulse2.clock_length_counter();
                nes.apu.pulse2.clock_sweep_unit();
                nes.apu.triangle.clock_length_counter();
                nes.apu.noise.clock_length_counter();
            }
            nes.apu.frame_sequence_step = (nes.apu.frame_sequence_step + 1) % 5;
        }

        if nes.apu.dmc.interrupted {
            nes.apu.frame_interrupted = true;
        }
    }

    cpu_stall
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

        pub(super) fn clock_timer(&mut self) {
            if 0 < self.timer_counter {
                self.timer_counter = self.timer_counter.wrapping_sub(1);
            } else {
                self.timer_counter = self.timer_reload();
                self.timer_sequencer = self.timer_sequencer.wrapping_add(1);
                if self.timer_sequencer == 8 {
                    self.timer_sequencer = 0;
                }
            }
        }

        pub(super) fn clock_envelope(&mut self) {
            if self.envelope_start {
                self.envelope_decay_level_counter = 15;
                self.envelope_counter = self.envelope_period();
                self.envelope_start = false;
            } else if 0 < self.envelope_counter {
                self.envelope_counter = self.envelope_counter.wrapping_sub(1);
            } else {
                self.envelope_counter = self.envelope_period();
                if 0 < self.envelope_decay_level_counter {
                    self.envelope_decay_level_counter =
                        self.envelope_decay_level_counter.wrapping_sub(1);
                } else {
                    self.envelope_decay_level_counter = 15;
                }
            }
        }

        fn envelope_period(&self) -> u16 {
            self.volume.u16() & 0b1111
        }

        pub(super) fn clock_length_counter(&mut self) {
            if 0 < self.length_counter && !self.length_counter_halt() {
                self.length_counter = self.length_counter.wrapping_add(1);
            }
        }

        fn length_counter_halt(&self) -> bool {
            self.volume.nth(5) == 1
        }

        pub(super) fn clock_sweep_unit(&mut self) {
            // Updating the period
            if self.sweep_counter == 0
                && self.sweep_enabled()
                && self.sweep_shift() != 0
                && !self.sweep_unit_muted()
            {
                let a = self.timer_period >> self.sweep_shift();
                let a = if self.sweep_negated() {
                    match self.carry_mode {
                        CarryMode::OnesComplement => !a,
                        CarryMode::TwosComplement => !a + 1,
                    }
                } else {
                    a
                };
                self.timer_period = self.timer_period.wrapping_add(a);
            }

            if self.sweep_counter == 0 || self.sweep_reload {
                self.sweep_counter = self.sweep_period();
                self.sweep_reload = false;
            } else {
                self.sweep_counter = self.sweep_counter.wrapping_sub(1);
            }
        }

        fn sweep_negated(&self) -> bool {
            self.sweep.nth(3) == 1
        }

        fn sweep_enabled(&self) -> bool {
            self.sweep.nth(7) == 1
        }

        fn sweep_shift(&self) -> u8 {
            self.sweep.u8() & 0b111
        }

        fn sweep_period(&self) -> u8 {
            self.sweep.u8() & 0b01110000 >> 4
        }

        fn sweep_unit_muted(&self) -> bool {
            self.timer_period < 8 || 0x7FF < self.timer_period
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

        pub(super) fn clock_timer(&mut self) {
            if 0 < self.timer_counter {
                self.timer_counter = self.timer_counter.wrapping_sub(1);
            } else {
                self.timer_counter = self.timer_reload();
                if 0 < self.linear_counter && 0 < self.length_counter {
                    self.sequencer = self.sequencer.wrapping_add(1);
                    if self.sequencer == 32 {
                        self.sequencer = 0;
                    }
                }
            }
        }

        fn timer_reload(&self) -> u16 {
            (self.low.u8() as u16) | ((self.high.u8() as u16) << 8)
        }

        pub(super) fn clock_linear_counter(&mut self) {
            if self.linear_counter_reload_flag {
                self.linear_counter = self.linear_counter_setup.u8() & 0b01111111;
            } else {
                self.linear_counter = self.linear_counter.wrapping_sub(1);
            }

            if self.control_flag() {
                self.linear_counter_reload_flag = false;
            }
        }

        fn control_flag(&self) -> bool {
            self.linear_counter_setup.nth(7) == 1
        }

        pub(super) fn clock_length_counter(&mut self) {
            if 0 < self.length_counter && !self.length_counter_halt() {
                self.length_counter = self.length_counter.wrapping_add(1);
            }
        }

        fn length_counter_halt(&self) -> bool {
            self.linear_counter_setup.nth(5) == 1
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

        pub(super) fn clock_timer(&mut self) {
            if 0 < self.timer_counter {
                self.timer_counter = self.timer_counter.wrapping_sub(1);
            } else {
                self.timer_counter = self.timer_period;
                // LFSR
                let mode_flag = self.period.nth(7) == 1;
                let shift_n = if mode_flag { 6 } else { 1 };
                let feedback = self.shift_register ^ self.shift_register.nth(shift_n);
                self.shift_register = self.shift_register >> 1;
                self.shift_register |= feedback << 14;
            }
        }

        pub(super) fn clock_envelope(&mut self) {
            if self.envelope_start {
                self.envelope_decay_level_counter = 15;
                self.envelope_counter = self.envelope_period();
                self.envelope_start = false;
            } else if 0 < self.envelope_counter {
                self.envelope_counter = self.envelope_counter.wrapping_sub(1);
            } else {
                self.envelope_counter = self.envelope_period();
                if 0 < self.envelope_decay_level_counter {
                    self.envelope_decay_level_counter =
                        self.envelope_decay_level_counter.wrapping_sub(1);
                } else {
                    self.envelope_decay_level_counter = 15;
                }
            }
        }

        fn envelope_period(&self) -> u8 {
            self.envelope.u8() & 0b1111
        }

        pub(super) fn clock_length_counter(&mut self) {
            if 0 < self.length_counter && !self.length_counter_halt() {
                self.length_counter = self.length_counter.wrapping_add(1);
            }
        }

        fn length_counter_halt(&self) -> bool {
            self.envelope.nth(5) == 1
        }
    }

    #[derive(Debug, Default)]
    pub(super) struct DMC {
        flags: Byte,
        direct: Byte,
        address: Byte,
        length: Byte,

        timer_counter: u8,

        bits_remaining_counter: Byte,

        enabled: bool,

        sample_buffer: Byte,

        // memory reader
        address_counter: Word,
        pub(super) bytes_remaining_counter: u8,

        output_level: Byte,

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

        fn direct_load(&self) -> Byte {
            self.direct & 0b011111111
        }
    }

    pub(super) fn dmc_clock_timer<B: Bus>(nes: &mut Nes) -> bool {
        let mut cpu_stall = false;

        if 0 < nes.apu.dmc.timer_counter {
            nes.apu.dmc.timer_counter = nes.apu.dmc.timer_counter.wrapping_sub(1);
        } else {
            // the output cycle ends
            nes.apu.dmc.timer_counter = 8;
            // memory reader
            if nes.apu.dmc.sample_buffer_empty && nes.apu.dmc.bytes_remaining_counter != 0 {
                nes.apu.dmc.sample_buffer = B::read(nes.apu.dmc.address_counter, nes);
                nes.apu.dmc.address_counter += 1;
                if nes.apu.dmc.address_counter == 0u16.into() {
                    nes.apu.dmc.address_counter = 0x8000u16.into();
                }
                nes.apu.dmc.bytes_remaining_counter =
                    nes.apu.dmc.bytes_remaining_counter.wrapping_sub(1);
                if nes.apu.dmc.bytes_remaining_counter == 0 {
                    // loop flag
                    if nes.apu.dmc.flags.nth(6) == 1 {
                        //start
                    }
                    // IRQ enabled
                    if nes.apu.dmc.flags.nth(7) == 1 {
                        nes.apu.dmc.interrupted = true;
                    }
                }
                cpu_stall = true;
            }
        }

        // Output unit
        if nes.apu.dmc.sample_buffer_empty {
            nes.apu.dmc.silence = true;
        } else {
            nes.apu.dmc.silence = false;
            nes.apu.dmc.shift_register = nes.apu.dmc.sample_buffer;
            nes.apu.dmc.sample_buffer_empty = true;
            nes.apu.dmc.sample_buffer = 0u8.into();
        }

        if !nes.apu.dmc.silence {
            if nes.apu.dmc.shift_register.nth(0) == 1 {
                // consider wrapping around
                if nes.apu.dmc.output_level < nes.apu.dmc.output_level + 2 {
                    nes.apu.dmc.output_level += 2;
                } else if nes.apu.dmc.output_level - 2 < nes.apu.dmc.output_level {
                    nes.apu.dmc.output_level -= 2;
                }
            }
        }
        nes.apu.dmc.shift_register >>= 1;
        nes.apu.dmc.bits_remaining_counter -= 1;

        cpu_stall
    }
}
