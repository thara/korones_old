use crate::bus::*;
use crate::data_unit::*;
use crate::nes::*;

#[derive(Debug, Default, Clone)]
pub struct Cpu {
    pub(crate) a: Byte,
    pub(crate) x: Byte,
    pub(crate) y: Byte,
    pub(crate) s: Byte,
    pub(crate) p: Status,
    pub(crate) pc: Word,

    pub(crate) cycles: u128,
}

bitflags! {
    #[derive(Default)]
    pub(crate) struct Status: u8 {
        // Negative
        const N = 1 << 7;
        // Overflow
        const V = 1 << 6;
        const R = 1 << 5;
        const B = 1 << 4;
        // Decimal mode
        const D = 1 << 3;
        // IRQ prevention
        const I = 1 << 2;
        // Zero
        const Z = 1 << 1;
        // Carry
        const C = 1 << 0;
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        const OPERATED_B = 0b110000;
        const INTERRUPTED_B = 0b100000;
    }
}

impl Nes {
    pub fn power_on(&mut self) {
        // https://wiki.nesdev.com/w/index.php/CPU_power_up_state

        // IRQ disabled
        self.cpu.p = Status::from_bits_truncate(0x34);
        self.cpu.a = 0x00.into();
        self.cpu.x = 0x00.into();
        self.cpu.y = 0x00.into();
        self.cpu.s = 0xFD.into();
        // frame irq disabled
        self.write(0x4017, 0x00);
        // all channels disabled
        self.write(0x4015, 0x00);

        for a in 0x4000..=0x400F {
            self.write(a, 0x00);
        }
        for a in 0x4010..=0x4013 {
            self.write(a, 0x00);
        }
    }
}

pub fn step(nes: &mut Nes) -> u128 {
    let before = nes.cpu.cycles;

    // fetch
    let opcode = nes.read(nes.cpu.pc);
    nes.cpu.pc += 1;

    let instruction = decode(opcode);
    let (_, addressing_mode) = instruction;

    // get operand
    let operand = match addressing_mode {
        AddressingMode::Implicit => Word::from(0x00u16),
        AddressingMode::Accumulator => nes.cpu.a.into(),
        AddressingMode::Immediate => {
            let operand = nes.cpu.pc;
            nes.cpu.pc += 1;
            operand
        }
        AddressingMode::ZeroPage => {
            let operand = Word::from(nes.read(nes.cpu.pc)) & 0xFF;
            nes.cpu.pc += 1;
            operand
        }
        AddressingMode::ZeroPageX => {
            let operand = (Word::from(nes.read(nes.cpu.pc)) + Word::from(nes.cpu.x)) & 0xFF;
            nes.cpu.pc += 1;
            nes.cpu.cycles += 1;
            operand
        }
        AddressingMode::ZeroPageY => {
            let operand = (Word::from(nes.read(nes.cpu.pc)) + Word::from(nes.cpu.y)) & 0xFF;
            nes.cpu.pc += 1;
            nes.cpu.cycles += 1;
            operand
        }
        AddressingMode::Absolute => {
            let operand = nes.read_word(nes.cpu.pc);
            nes.cpu.pc += 2;
            operand
        }
        AddressingMode::AbsoluteX { penalty } => {
            let data = nes.read_word(nes.cpu.pc);
            let operand = data + Word::from(nes.cpu.x);
            nes.cpu.pc += 2;
            if penalty {
                if page_crossed_u16(nes.cpu.x, data) {
                    nes.cpu.cycles += 1;
                }
            } else {
                nes.cpu.cycles += 1;
            }
            operand
        }
        AddressingMode::AbsoluteY { penalty } => {
            let data = nes.read_word(nes.cpu.pc);
            let operand = data + Word::from(nes.cpu.y);
            nes.cpu.pc += 2;
            if penalty {
                if page_crossed_u16(nes.cpu.y, data) {
                    nes.cpu.cycles += 1;
                }
            } else {
                nes.cpu.cycles += 1;
            }
            operand
        }
        AddressingMode::Relative => {
            let operand: Word = nes.read(nes.cpu.pc).into();
            nes.cpu.pc += 1;
            operand
        }
        AddressingMode::Indirect => {
            let data = nes.read_word(nes.cpu.pc);
            let operand = nes.read_on_indirect(data);
            nes.cpu.pc += 2;
            operand
        }
        AddressingMode::IndexedIndirect => {
            let data = nes.read(nes.cpu.pc);
            let operand = nes.read_on_indirect(Word::from(data + nes.cpu.x) & 0xFF);
            nes.cpu.pc += 1;
            nes.cpu.cycles += 1;
            operand
        }
        AddressingMode::IndirectIndexed => {
            let y: Word = nes.cpu.y.into();
            let data: Word = nes.read(nes.cpu.pc).into();
            let operand = nes.read_on_indirect(data) + y;
            nes.cpu.pc += 1;
            if page_crossed_u16(y, operand - y) {
                nes.cpu.cycles += 1;
            }
            operand
        }
    };

    // execute
    match instruction {
        (Mnemonic::LDA, _) => nes.lda(operand),
        (Mnemonic::LDX, _) => nes.ldx(operand),
        (Mnemonic::LDY, _) => nes.ldy(operand),
        (Mnemonic::STA, AddressingMode::IndirectIndexed) => {
            nes.sta(operand);
            nes.cpu.cycles += 1;
        }
        (Mnemonic::STA, _) => nes.sta(operand),
        (Mnemonic::STX, _) => nes.stx(operand),
        (Mnemonic::STY, _) => nes.sty(operand),
        (Mnemonic::TAX, _) => nes.tax(operand),
        (Mnemonic::TSX, _) => nes.tsx(operand),
        (Mnemonic::TAY, _) => nes.tay(operand),
        (Mnemonic::TXA, _) => nes.txa(operand),
        (Mnemonic::TXS, _) => nes.txs(operand),
        (Mnemonic::TYA, _) => nes.tya(operand),
        (Mnemonic::PHA, _) => nes.pha(operand),
        (Mnemonic::PHP, _) => nes.php(operand),
        (Mnemonic::PLA, _) => nes.pla(operand),
        (Mnemonic::PLP, _) => nes.plp(operand),
        (Mnemonic::AND, _) => nes.and(operand),
        (Mnemonic::EOR, _) => nes.eor(operand),
        (Mnemonic::ORA, _) => nes.ora(operand),
        (Mnemonic::BIT, _) => nes.bit(operand),
        (Mnemonic::ADC, _) => nes.adc(operand),
        (Mnemonic::SBC, _) => nes.sbc(operand),
        (Mnemonic::CMP, _) => nes.cmp(operand),
        (Mnemonic::CPX, _) => nes.cpx(operand),
        (Mnemonic::CPY, _) => nes.cpy(operand),
        (Mnemonic::INC, _) => nes.inc(operand),
        (Mnemonic::INX, _) => nes.inx(operand),
        (Mnemonic::INY, _) => nes.iny(operand),
        (Mnemonic::DEC, _) => nes.dec(operand),
        (Mnemonic::DEX, _) => nes.dex(operand),
        (Mnemonic::DEY, _) => nes.dey(operand),
        (Mnemonic::ASL, AddressingMode::Accumulator) => nes.asl_for_accumelator(operand),
        (Mnemonic::ASL, _) => nes.asl(operand),
        (Mnemonic::LSR, AddressingMode::Accumulator) => nes.lsr_for_accumelator(operand),
        (Mnemonic::LSR, _) => nes.lsr(operand),
        (Mnemonic::ROL, AddressingMode::Accumulator) => nes.rol_for_accumelator(operand),
        (Mnemonic::ROL, _) => nes.rol(operand),
        (Mnemonic::ROR, AddressingMode::Accumulator) => nes.ror_for_accumelator(operand),
        (Mnemonic::ROR, _) => nes.ror(operand),
        (Mnemonic::JMP, _) => nes.jmp(operand),
        (Mnemonic::JSR, _) => nes.jsr(operand),
        (Mnemonic::RTS, _) => nes.rts(operand),
        (Mnemonic::RTI, _) => nes.rti(operand),
        (Mnemonic::BCC, _) => nes.bcc(operand),
        (Mnemonic::BCS, _) => nes.bcs(operand),
        (Mnemonic::BEQ, _) => nes.beq(operand),
        (Mnemonic::BMI, _) => nes.bmi(operand),
        (Mnemonic::BNE, _) => nes.bne(operand),
        (Mnemonic::BPL, _) => nes.bpl(operand),
        (Mnemonic::BVC, _) => nes.bvc(operand),
        (Mnemonic::BVS, _) => nes.bvs(operand),
        (Mnemonic::CLC, _) => nes.clc(operand),
        (Mnemonic::CLD, _) => nes.cld(operand),
        (Mnemonic::CLI, _) => nes.cli(operand),
        (Mnemonic::CLV, _) => nes.clv(operand),
        (Mnemonic::SEC, _) => nes.sec(operand),
        (Mnemonic::SED, _) => nes.sed(operand),
        (Mnemonic::SEI, _) => nes.sei(operand),
        (Mnemonic::BRK, _) => nes.brk(operand),
        (Mnemonic::NOP, _) => nes.nop(operand),
        (Mnemonic::LAX, _) => nes.lax(operand),
        (Mnemonic::SAX, _) => nes.sax(operand),
        (Mnemonic::DCP, _) => nes.dcp(operand),
        (Mnemonic::ISB, _) => nes.isb(operand),
        (Mnemonic::SLO, _) => nes.slo(operand),
        (Mnemonic::RLA, _) => nes.rla(operand),
        (Mnemonic::SRE, _) => nes.sre(operand),
        (Mnemonic::RRA, _) => nes.rra(operand),
    }

    if before <= nes.cpu.cycles {
        nes.cpu.cycles - before
    } else {
        u128::MAX - before + nes.cpu.cycles
    }
}

pub(crate) fn decode(opcode: Byte) -> (Mnemonic, AddressingMode) {
    match opcode.u8() {
        0xA9 => (Mnemonic::LDA, AddressingMode::Immediate),
        0xA5 => (Mnemonic::LDA, AddressingMode::ZeroPage),
        0xB5 => (Mnemonic::LDA, AddressingMode::ZeroPageX),
        0xAD => (Mnemonic::LDA, AddressingMode::Absolute),
        0xBD => (Mnemonic::LDA, AddressingMode::AbsoluteX { penalty: true }),
        0xB9 => (Mnemonic::LDA, AddressingMode::AbsoluteY { penalty: true }),
        0xA1 => (Mnemonic::LDA, AddressingMode::IndexedIndirect),
        0xB1 => (Mnemonic::LDA, AddressingMode::IndirectIndexed),
        0xA2 => (Mnemonic::LDX, AddressingMode::Immediate),
        0xA6 => (Mnemonic::LDX, AddressingMode::ZeroPage),
        0xB6 => (Mnemonic::LDX, AddressingMode::ZeroPageY),
        0xAE => (Mnemonic::LDX, AddressingMode::Absolute),
        0xBE => (Mnemonic::LDX, AddressingMode::AbsoluteY { penalty: true }),
        0xA0 => (Mnemonic::LDY, AddressingMode::Immediate),
        0xA4 => (Mnemonic::LDY, AddressingMode::ZeroPage),
        0xB4 => (Mnemonic::LDY, AddressingMode::ZeroPageX),
        0xAC => (Mnemonic::LDY, AddressingMode::Absolute),
        0xBC => (Mnemonic::LDY, AddressingMode::AbsoluteX { penalty: true }),
        0x85 => (Mnemonic::STA, AddressingMode::ZeroPage),
        0x95 => (Mnemonic::STA, AddressingMode::ZeroPageX),
        0x8D => (Mnemonic::STA, AddressingMode::Absolute),
        0x9D => (Mnemonic::STA, AddressingMode::AbsoluteX { penalty: false }),
        0x99 => (Mnemonic::STA, AddressingMode::AbsoluteY { penalty: false }),
        0x81 => (Mnemonic::STA, AddressingMode::IndexedIndirect),
        0x91 => (Mnemonic::STA, AddressingMode::IndirectIndexed),
        0x86 => (Mnemonic::STX, AddressingMode::ZeroPage),
        0x96 => (Mnemonic::STX, AddressingMode::ZeroPageY),
        0x8E => (Mnemonic::STX, AddressingMode::Absolute),
        0x84 => (Mnemonic::STY, AddressingMode::ZeroPage),
        0x94 => (Mnemonic::STY, AddressingMode::ZeroPageX),
        0x8C => (Mnemonic::STY, AddressingMode::Absolute),
        0xAA => (Mnemonic::TAX, AddressingMode::Implicit),
        0xBA => (Mnemonic::TSX, AddressingMode::Implicit),
        0xA8 => (Mnemonic::TAY, AddressingMode::Implicit),
        0x8A => (Mnemonic::TXA, AddressingMode::Implicit),
        0x9A => (Mnemonic::TXS, AddressingMode::Implicit),
        0x98 => (Mnemonic::TYA, AddressingMode::Implicit),

        0x48 => (Mnemonic::PHA, AddressingMode::Implicit),
        0x08 => (Mnemonic::PHP, AddressingMode::Implicit),
        0x68 => (Mnemonic::PLA, AddressingMode::Implicit),
        0x28 => (Mnemonic::PLP, AddressingMode::Implicit),

        0x29 => (Mnemonic::AND, AddressingMode::Immediate),
        0x25 => (Mnemonic::AND, AddressingMode::ZeroPage),
        0x35 => (Mnemonic::AND, AddressingMode::ZeroPageX),
        0x2D => (Mnemonic::AND, AddressingMode::Absolute),
        0x3D => (Mnemonic::AND, AddressingMode::AbsoluteX { penalty: true }),
        0x39 => (Mnemonic::AND, AddressingMode::AbsoluteY { penalty: true }),
        0x21 => (Mnemonic::AND, AddressingMode::IndexedIndirect),
        0x31 => (Mnemonic::AND, AddressingMode::IndirectIndexed),
        0x49 => (Mnemonic::EOR, AddressingMode::Immediate),
        0x45 => (Mnemonic::EOR, AddressingMode::ZeroPage),
        0x55 => (Mnemonic::EOR, AddressingMode::ZeroPageX),
        0x4D => (Mnemonic::EOR, AddressingMode::Absolute),
        0x5D => (Mnemonic::EOR, AddressingMode::AbsoluteX { penalty: true }),
        0x59 => (Mnemonic::EOR, AddressingMode::AbsoluteY { penalty: true }),
        0x41 => (Mnemonic::EOR, AddressingMode::IndexedIndirect),
        0x51 => (Mnemonic::EOR, AddressingMode::IndirectIndexed),
        0x09 => (Mnemonic::ORA, AddressingMode::Immediate),
        0x05 => (Mnemonic::ORA, AddressingMode::ZeroPage),
        0x15 => (Mnemonic::ORA, AddressingMode::ZeroPageX),
        0x0D => (Mnemonic::ORA, AddressingMode::Absolute),
        0x1D => (Mnemonic::ORA, AddressingMode::AbsoluteX { penalty: true }),
        0x19 => (Mnemonic::ORA, AddressingMode::AbsoluteY { penalty: true }),
        0x01 => (Mnemonic::ORA, AddressingMode::IndexedIndirect),
        0x11 => (Mnemonic::ORA, AddressingMode::IndirectIndexed),
        0x24 => (Mnemonic::BIT, AddressingMode::ZeroPage),
        0x2C => (Mnemonic::BIT, AddressingMode::Absolute),

        0x69 => (Mnemonic::ADC, AddressingMode::Immediate),
        0x65 => (Mnemonic::ADC, AddressingMode::ZeroPage),
        0x75 => (Mnemonic::ADC, AddressingMode::ZeroPageX),
        0x6D => (Mnemonic::ADC, AddressingMode::Absolute),
        0x7D => (Mnemonic::ADC, AddressingMode::AbsoluteX { penalty: true }),
        0x79 => (Mnemonic::ADC, AddressingMode::AbsoluteY { penalty: true }),
        0x61 => (Mnemonic::ADC, AddressingMode::IndexedIndirect),
        0x71 => (Mnemonic::ADC, AddressingMode::IndirectIndexed),
        0xE9 => (Mnemonic::SBC, AddressingMode::Immediate),
        0xE5 => (Mnemonic::SBC, AddressingMode::ZeroPage),
        0xF5 => (Mnemonic::SBC, AddressingMode::ZeroPageX),
        0xED => (Mnemonic::SBC, AddressingMode::Absolute),
        0xFD => (Mnemonic::SBC, AddressingMode::AbsoluteX { penalty: true }),
        0xF9 => (Mnemonic::SBC, AddressingMode::AbsoluteY { penalty: true }),
        0xE1 => (Mnemonic::SBC, AddressingMode::IndexedIndirect),
        0xF1 => (Mnemonic::SBC, AddressingMode::IndirectIndexed),
        0xC9 => (Mnemonic::CMP, AddressingMode::Immediate),
        0xC5 => (Mnemonic::CMP, AddressingMode::ZeroPage),
        0xD5 => (Mnemonic::CMP, AddressingMode::ZeroPageX),
        0xCD => (Mnemonic::CMP, AddressingMode::Absolute),
        0xDD => (Mnemonic::CMP, AddressingMode::AbsoluteX { penalty: true }),
        0xD9 => (Mnemonic::CMP, AddressingMode::AbsoluteY { penalty: true }),
        0xC1 => (Mnemonic::CMP, AddressingMode::IndexedIndirect),
        0xD1 => (Mnemonic::CMP, AddressingMode::IndirectIndexed),
        0xE0 => (Mnemonic::CPX, AddressingMode::Immediate),
        0xE4 => (Mnemonic::CPX, AddressingMode::ZeroPage),
        0xEC => (Mnemonic::CPX, AddressingMode::Absolute),
        0xC0 => (Mnemonic::CPY, AddressingMode::Immediate),
        0xC4 => (Mnemonic::CPY, AddressingMode::ZeroPage),
        0xCC => (Mnemonic::CPY, AddressingMode::Absolute),

        0xE6 => (Mnemonic::INC, AddressingMode::ZeroPage),
        0xF6 => (Mnemonic::INC, AddressingMode::ZeroPageX),
        0xEE => (Mnemonic::INC, AddressingMode::Absolute),
        0xFE => (Mnemonic::INC, AddressingMode::AbsoluteX { penalty: false }),
        0xE8 => (Mnemonic::INX, AddressingMode::Implicit),
        0xC8 => (Mnemonic::INY, AddressingMode::Implicit),
        0xC6 => (Mnemonic::DEC, AddressingMode::ZeroPage),
        0xD6 => (Mnemonic::DEC, AddressingMode::ZeroPageX),
        0xCE => (Mnemonic::DEC, AddressingMode::Absolute),
        0xDE => (Mnemonic::DEC, AddressingMode::AbsoluteX { penalty: false }),
        0xCA => (Mnemonic::DEX, AddressingMode::Implicit),
        0x88 => (Mnemonic::DEY, AddressingMode::Implicit),

        0x0A => (Mnemonic::ASL, AddressingMode::Accumulator),
        0x06 => (Mnemonic::ASL, AddressingMode::ZeroPage),
        0x16 => (Mnemonic::ASL, AddressingMode::ZeroPageX),
        0x0E => (Mnemonic::ASL, AddressingMode::Absolute),
        0x1E => (Mnemonic::ASL, AddressingMode::AbsoluteX { penalty: false }),
        0x4A => (Mnemonic::LSR, AddressingMode::Accumulator),
        0x46 => (Mnemonic::LSR, AddressingMode::ZeroPage),
        0x56 => (Mnemonic::LSR, AddressingMode::ZeroPageX),
        0x4E => (Mnemonic::LSR, AddressingMode::Absolute),
        0x5E => (Mnemonic::LSR, AddressingMode::AbsoluteX { penalty: false }),
        0x2A => (Mnemonic::ROL, AddressingMode::Accumulator),
        0x26 => (Mnemonic::ROL, AddressingMode::ZeroPage),
        0x36 => (Mnemonic::ROL, AddressingMode::ZeroPageX),
        0x2E => (Mnemonic::ROL, AddressingMode::Absolute),
        0x3E => (Mnemonic::ROL, AddressingMode::AbsoluteX { penalty: false }),
        0x6A => (Mnemonic::ROR, AddressingMode::Accumulator),
        0x66 => (Mnemonic::ROR, AddressingMode::ZeroPage),
        0x76 => (Mnemonic::ROR, AddressingMode::ZeroPageX),
        0x6E => (Mnemonic::ROR, AddressingMode::Absolute),
        0x7E => (Mnemonic::ROR, AddressingMode::AbsoluteX { penalty: false }),

        0x4C => (Mnemonic::JMP, AddressingMode::Absolute),
        0x6C => (Mnemonic::JMP, AddressingMode::Indirect),
        0x20 => (Mnemonic::JSR, AddressingMode::Absolute),
        0x60 => (Mnemonic::RTS, AddressingMode::Implicit),
        0x40 => (Mnemonic::RTI, AddressingMode::Implicit),

        0x90 => (Mnemonic::BCC, AddressingMode::Relative),
        0xB0 => (Mnemonic::BCS, AddressingMode::Relative),
        0xF0 => (Mnemonic::BEQ, AddressingMode::Relative),
        0x30 => (Mnemonic::BMI, AddressingMode::Relative),
        0xD0 => (Mnemonic::BNE, AddressingMode::Relative),
        0x10 => (Mnemonic::BPL, AddressingMode::Relative),
        0x50 => (Mnemonic::BVC, AddressingMode::Relative),
        0x70 => (Mnemonic::BVS, AddressingMode::Relative),

        0x18 => (Mnemonic::CLC, AddressingMode::Implicit),
        0xD8 => (Mnemonic::CLD, AddressingMode::Implicit),
        0x58 => (Mnemonic::CLI, AddressingMode::Implicit),
        0xB8 => (Mnemonic::CLV, AddressingMode::Implicit),

        0x38 => (Mnemonic::SEC, AddressingMode::Implicit),
        0xF8 => (Mnemonic::SED, AddressingMode::Implicit),
        0x78 => (Mnemonic::SEI, AddressingMode::Implicit),

        0x00 => (Mnemonic::BRK, AddressingMode::Implicit),

        // Undocumented
        0xEB => (Mnemonic::SBC, AddressingMode::Immediate),

        0x04 | 0x44 | 0x64 => (Mnemonic::NOP, AddressingMode::ZeroPage),
        0x0C => (Mnemonic::NOP, AddressingMode::Absolute),
        0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 => (Mnemonic::NOP, AddressingMode::ZeroPageX),
        0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xEA | 0xFA => (Mnemonic::NOP, AddressingMode::Implicit),
        0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
            (Mnemonic::NOP, AddressingMode::AbsoluteX { penalty: true })
        }
        0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 => (Mnemonic::NOP, AddressingMode::Immediate),

        0xA3 => (Mnemonic::LAX, AddressingMode::IndexedIndirect),
        0xA7 => (Mnemonic::LAX, AddressingMode::ZeroPage),
        0xAF => (Mnemonic::LAX, AddressingMode::Absolute),
        0xB3 => (Mnemonic::LAX, AddressingMode::IndirectIndexed),
        0xB7 => (Mnemonic::LAX, AddressingMode::ZeroPageY),
        0xBF => (Mnemonic::LAX, AddressingMode::AbsoluteY { penalty: true }),

        0x83 => (Mnemonic::SAX, AddressingMode::IndexedIndirect),
        0x87 => (Mnemonic::SAX, AddressingMode::ZeroPage),
        0x8F => (Mnemonic::SAX, AddressingMode::Absolute),
        0x97 => (Mnemonic::SAX, AddressingMode::ZeroPageY),

        0xC3 => (Mnemonic::DCP, AddressingMode::IndexedIndirect),
        0xC7 => (Mnemonic::DCP, AddressingMode::ZeroPage),
        0xCF => (Mnemonic::DCP, AddressingMode::Absolute),
        0xD3 => (Mnemonic::DCP, AddressingMode::IndirectIndexed),
        0xD7 => (Mnemonic::DCP, AddressingMode::ZeroPageX),
        0xDB => (Mnemonic::DCP, AddressingMode::AbsoluteY { penalty: false }),
        0xDF => (Mnemonic::DCP, AddressingMode::AbsoluteX { penalty: false }),

        0xE3 => (Mnemonic::ISB, AddressingMode::IndexedIndirect),
        0xE7 => (Mnemonic::ISB, AddressingMode::ZeroPage),
        0xEF => (Mnemonic::ISB, AddressingMode::Absolute),
        0xF3 => (Mnemonic::ISB, AddressingMode::IndirectIndexed),
        0xF7 => (Mnemonic::ISB, AddressingMode::ZeroPageX),
        0xFB => (Mnemonic::ISB, AddressingMode::AbsoluteY { penalty: false }),
        0xFF => (Mnemonic::ISB, AddressingMode::AbsoluteX { penalty: false }),

        0x03 => (Mnemonic::SLO, AddressingMode::IndexedIndirect),
        0x07 => (Mnemonic::SLO, AddressingMode::ZeroPage),
        0x0F => (Mnemonic::SLO, AddressingMode::Absolute),
        0x13 => (Mnemonic::SLO, AddressingMode::IndirectIndexed),
        0x17 => (Mnemonic::SLO, AddressingMode::ZeroPageX),
        0x1B => (Mnemonic::SLO, AddressingMode::AbsoluteY { penalty: false }),
        0x1F => (Mnemonic::SLO, AddressingMode::AbsoluteX { penalty: false }),

        0x23 => (Mnemonic::RLA, AddressingMode::IndexedIndirect),
        0x27 => (Mnemonic::RLA, AddressingMode::ZeroPage),
        0x2F => (Mnemonic::RLA, AddressingMode::Absolute),
        0x33 => (Mnemonic::RLA, AddressingMode::IndirectIndexed),
        0x37 => (Mnemonic::RLA, AddressingMode::ZeroPageX),
        0x3B => (Mnemonic::RLA, AddressingMode::AbsoluteY { penalty: false }),
        0x3F => (Mnemonic::RLA, AddressingMode::AbsoluteX { penalty: false }),

        0x43 => (Mnemonic::SRE, AddressingMode::IndexedIndirect),
        0x47 => (Mnemonic::SRE, AddressingMode::ZeroPage),
        0x4F => (Mnemonic::SRE, AddressingMode::Absolute),
        0x53 => (Mnemonic::SRE, AddressingMode::IndirectIndexed),
        0x57 => (Mnemonic::SRE, AddressingMode::ZeroPageX),
        0x5B => (Mnemonic::SRE, AddressingMode::AbsoluteY { penalty: false }),
        0x5F => (Mnemonic::SRE, AddressingMode::AbsoluteX { penalty: false }),

        0x63 => (Mnemonic::RRA, AddressingMode::IndexedIndirect),
        0x67 => (Mnemonic::RRA, AddressingMode::ZeroPage),
        0x6F => (Mnemonic::RRA, AddressingMode::Absolute),
        0x73 => (Mnemonic::RRA, AddressingMode::IndirectIndexed),
        0x77 => (Mnemonic::RRA, AddressingMode::ZeroPageX),
        0x7B => (Mnemonic::RRA, AddressingMode::AbsoluteY { penalty: false }),
        0x7F => (Mnemonic::RRA, AddressingMode::AbsoluteX { penalty: false }),

        _ => (Mnemonic::NOP, AddressingMode::Implicit),
    }
}

impl Bus for Nes {
    fn read(&mut self, addr: impl Into<Word>) -> Byte {
        self.cpu.cycles += 1;
        self.read_bus(addr)
    }

    fn write(&mut self, addr: impl Into<Word>, value: impl Into<Byte>) {
        let addr = addr.into();
        let a: u16 = addr.into();
        let value = value.into();
        let v: u16 = value.into();

        if a == 0x4014u16 {
            // OAMDMA
            let start: u16 = v * 0x100u16;
            for a in start..(start + 0xFF) {
                let data = self.read_bus(a);
                self.cpu.cycles += 1;
                self.write_bus(0x2004u16, data);
                self.cpu.cycles += 1;
            }
            // dummy cycles
            self.cpu.cycles += 1;
            if self.cpu.cycles % 2 == 1 {
                self.cpu.cycles += 1;
            }
            return;
        }
        self.cpu.cycles += 1;
        self.write_bus(addr, value);
    }
}

// http://wiki.nesdev.com/w/index.php/CPU_addressing_modes
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[rustfmt::skip]
pub(crate) enum AddressingMode {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage, ZeroPageX, ZeroPageY,
    Absolute,
    AbsoluteX { penalty: bool },
    AbsoluteY { penalty: bool },
    Relative,
    Indirect, IndexedIndirect, IndirectIndexed,
}

// http://obelisk.me.uk/6502/reference.html
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[rustfmt::skip]
pub(crate) enum Mnemonic {
    // Load/Store Operations
    LDA, LDX, LDY, STA, STX, STY,
    // Register Operations
    TAX, TSX, TAY, TXA, TXS, TYA,
    // Stack instructions
    PHA, PHP, PLA, PLP,
    // Logical instructions
    AND, EOR, ORA, BIT,
    // Arithmetic instructions
    ADC, SBC, CMP, CPX, CPY,
    // Increment/Decrement instructions
    INC, INX, INY, DEC, DEX, DEY,
    // Shift instructions
    ASL, LSR, ROL, ROR,
    // Jump instructions
    JMP, JSR, RTS, RTI,
    // Branch instructions
    BCC, BCS, BEQ, BMI, BNE, BPL, BVC, BVS,
    // Flag control instructions
    CLC, CLD, CLI, CLV, SEC, SED, SEI,
    // Misc
    BRK, NOP,
    // Unofficial
    LAX, SAX, DCP, ISB, SLO, RLA, SRE, RRA,
}

type Operand = Word;

mod instructions {
    use super::*;

    impl Nes {
        // LoaD Accumulator
        pub(super) fn lda(&mut self, operand: Operand) {
            self.cpu.a = self.read(operand);
            self.cpu.p.set_zn(self.cpu.a)
        }

        // LoaD X register
        pub(super) fn ldx(&mut self, operand: Operand) {
            self.cpu.x = self.read(operand);
            self.cpu.p.set_zn(self.cpu.x)
        }

        // LoaD Y register
        pub(super) fn ldy(&mut self, operand: Operand) {
            self.cpu.y = self.read(operand);
            self.cpu.p.set_zn(self.cpu.y)
        }

        // STore Accumulator
        pub(super) fn sta(&mut self, operand: Operand) {
            self.write(operand, self.cpu.a)
        }

        // STore X register
        pub(super) fn stx(&mut self, operand: Operand) {
            self.write(operand, self.cpu.x)
        }

        // STore Y register
        pub(super) fn sty(&mut self, operand: Operand) {
            self.write(operand, self.cpu.y)
        }

        // Transfer Accumulator to X
        pub(super) fn tax(&mut self, _: Operand) {
            self.cpu.x = self.cpu.a;
            self.cpu.p.set_zn(self.cpu.x);
            self.cpu.cycles += 1;
        }

        // Transfer Stack pointer to X
        pub(super) fn tsx(&mut self, _: Operand) {
            self.cpu.x = self.cpu.s;
            self.cpu.p.set_zn(self.cpu.x);
            self.cpu.cycles += 1;
        }

        // Transfer Accumulator to Y
        pub(super) fn tay(&mut self, _: Operand) {
            self.cpu.y = self.cpu.a;
            self.cpu.p.set_zn(self.cpu.y);
            self.cpu.cycles += 1;
        }

        // Transfer X to Accumulator
        pub(super) fn txa(&mut self, _: Operand) {
            self.cpu.a = self.cpu.x;
            self.cpu.p.set_zn(self.cpu.a);
            self.cpu.cycles += 1;
        }

        // Transfer X to Stack pointer
        pub(super) fn txs(&mut self, _: Operand) {
            self.cpu.s = self.cpu.x;
            self.cpu.cycles += 1;
        }

        // Transfer Y to Accumulator
        pub(super) fn tya(&mut self, _: Operand) {
            self.cpu.a = self.cpu.y;
            self.cpu.p.set_zn(self.cpu.a);
            self.cpu.cycles += 1;
        }

        // PusH Accumulator
        pub(super) fn pha(&mut self, _: Operand) {
            self.push_stack(self.cpu.a);
            self.cpu.cycles += 1;
        }

        // PusH Processor status
        pub(super) fn php(&mut self, _: Operand) {
            // https://wiki.selfdev.com/w/index.php/Statu_s_flags#The_B_flag
            // http://visual6502.org/wiki/index.php?titl_e=6502_BRK_and_B_bit
            self.push_stack((self.cpu.p | Status::OPERATED_B).bits().into());
            self.cpu.cycles += 1;
        }

        // PulL Accumulator
        pub(super) fn pla(&mut self, _: Operand) {
            self.cpu.a = self.pull_stack();
            self.cpu.p.set_zn(self.cpu.a);
            self.cpu.cycles += 1;
            self.cpu.cycles += 1;
        }

        // PulL Processor status
        pub(super) fn plp(&mut self, _: Operand) {
            // https://wiki.selfdev.com/w/index.php/Status_flags#The_B_flag
            // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
            self.cpu.p =
                Status::from_bits_truncate(self.pull_stack().into()) & !Status::B | Status::R;
            self.cpu.cycles += 1;
            self.cpu.cycles += 1;
        }

        // bitwise AND with accumulator
        pub(super) fn and(&mut self, operand: Operand) {
            let value = self.read(operand);
            self.cpu.a &= value;
            self.cpu.p.set_zn(self.cpu.a);
        }

        // bitwise Exclusive OR
        pub(super) fn eor(&mut self, operand: Operand) {
            let value = self.read(operand);
            self.cpu.a ^= value;
            self.cpu.p.set_zn(self.cpu.a);
        }

        // bitwise OR with Accumulator
        pub(super) fn ora(&mut self, operand: Operand) {
            let value = self.read(operand);
            self.cpu.a |= value;
            self.cpu.p.set_zn(self.cpu.a);
        }

        // test BITs
        pub(super) fn bit(&mut self, operand: Operand) {
            let value = self.read(operand);
            let data = self.cpu.a & value;
            self.cpu.p.set(Status::Z, data.u8() == 0);
            self.cpu.p.set(Status::V, value.nth(6) == 1);
            self.cpu.p.set(Status::N, value.nth(7) == 1);
        }

        // ADd with Carry
        pub(super) fn adc(&mut self, operand: Operand) {
            let a = self.cpu.a;
            let val = self.read(operand);
            let mut result = a + val;

            if self.cpu.p.contains(Status::C) {
                result += 1;
            }

            // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
            let a7 = a.nth(7);
            let v7 = val.nth(7);
            let c6 = a7 ^ v7 ^ result.nth(7);
            let c7 = (a7 & v7) | (a7 & c6) | (v7 & c6);

            self.cpu.p.set(Status::C, c7 == 1);
            self.cpu.p.set(Status::V, (c6 ^ c7) == 1);

            self.cpu.a = result;
            self.cpu.p.set_zn(self.cpu.a)
        }

        // SuBtract with carry
        pub(super) fn sbc(&mut self, operand: Operand) {
            let a = self.cpu.a;
            let val = !self.read(operand);
            let mut result = a + val;

            if self.cpu.p.contains(Status::C) {
                result += 1;
            }

            // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
            let a7 = a.nth(7);
            let v7 = val.nth(7);
            let c6 = a7 ^ v7 ^ result.nth(7);
            let c7 = (a7 & v7) | (a7 & c6) | (v7 & c6);

            self.cpu.p.set(Status::C, c7 == 1);
            self.cpu.p.set(Status::V, (c6 ^ c7) == 1);

            self.cpu.a = result;
            self.cpu.p.set_zn(self.cpu.a)
        }

        // CoMPare accumulator
        pub(super) fn cmp(&mut self, operand: Operand) {
            let cmp = Word::from(self.cpu.a) - Word::from(self.read(operand));
            let cmp_i16 = <Word as Into<i16>>::into(cmp);

            self.cpu.p.set(Status::C, 0 <= cmp_i16);
            self.cpu.p.set_zn(cmp_i16 as u16);
        }

        // ComPare X register
        pub(super) fn cpx(&mut self, operand: Operand) {
            let value = self.read(operand);
            let cmp = self.cpu.x - value;

            self.cpu.p.set(Status::C, value <= self.cpu.x);
            self.cpu.p.set_zn(cmp);
        }

        // ComPare Y register
        pub(super) fn cpy(&mut self, operand: Operand) {
            let value = self.read(operand);
            let cmp = self.cpu.y - value;

            self.cpu.p.set(Status::C, value <= self.cpu.y);
            self.cpu.p.set_zn(cmp);
        }

        // INCrement memory
        pub(super) fn inc(&mut self, operand: Operand) {
            let result = self.read(operand) + 1;

            self.cpu.p.set_zn(result);
            self.write(operand, result);
            self.cpu.cycles += 1
        }

        // INcrement X register
        pub(super) fn inx(&mut self, _: Operand) {
            self.cpu.x += 1;
            self.cpu.p.set_zn(self.cpu.x);
            self.cpu.cycles += 1
        }

        // INcrement Y register
        pub(super) fn iny(&mut self, _: Operand) {
            self.cpu.y += 1;
            self.cpu.p.set_zn(self.cpu.y);
            self.cpu.cycles += 1
        }

        // DECrement memory
        pub(super) fn dec(&mut self, operand: Operand) {
            let result = self.read(operand) - 1;

            self.cpu.p.set_zn(result);
            self.write(operand, result);
            self.cpu.cycles += 1
        }

        // DEcrement X register
        pub(super) fn dex(&mut self, _: Operand) {
            self.cpu.x -= 1;
            self.cpu.p.set_zn(self.cpu.x);
            self.cpu.cycles += 1
        }

        // DEcrement Y register
        pub(super) fn dey(&mut self, _: Operand) {
            self.cpu.y -= 1;
            self.cpu.p.set_zn(self.cpu.y);
            self.cpu.cycles += 1
        }

        // Arithmetic Shift Left
        pub(super) fn asl(&mut self, operand: Operand) {
            let mut data = self.read(operand);

            self.cpu.p.set(Status::C, data.nth(7) == 1);
            data <<= 1;
            self.cpu.p.set_zn(data);

            self.write(operand, data);
            self.cpu.cycles += 1;
        }

        pub(super) fn asl_for_accumelator(&mut self, _: Operand) {
            self.cpu.p.set(Status::C, self.cpu.a.nth(7) == 1);
            self.cpu.a <<= 1;
            self.cpu.p.set_zn(self.cpu.a);

            self.cpu.cycles += 1;
        }

        // Logical Shift Right
        pub(super) fn lsr(&mut self, operand: Operand) {
            let mut data = self.read(operand);

            self.cpu.p.set(Status::C, data.nth(0) == 1);
            data >>= 1;
            self.cpu.p.set_zn(data);

            self.write(operand, data);
            self.cpu.cycles += 1;
        }

        pub(super) fn lsr_for_accumelator(&mut self, _: Operand) {
            self.cpu.p.set(Status::C, self.cpu.a.nth(0) == 1);
            self.cpu.a >>= 1;
            self.cpu.p.set_zn(self.cpu.a);

            self.cpu.cycles += 1;
        }

        // ROtate Left
        pub(super) fn rol(&mut self, operand: Operand) {
            let mut data = self.read(operand);
            let c = data.nth(7);

            data <<= 1;
            if self.cpu.p.contains(Status::C) {
                data |= 0x01;
            }
            self.cpu.p.set(Status::C, c == 1);
            self.cpu.p.set_zn(data);
            self.write(operand, data);
            self.cpu.cycles += 1;
        }

        pub(super) fn rol_for_accumelator(&mut self, _: Operand) {
            let c = self.cpu.a.nth(7);

            let mut a = self.cpu.a << 1;
            if self.cpu.p.contains(Status::C) {
                a |= 0x01;
            }
            self.cpu.a = a;
            self.cpu.p.set(Status::C, c == 1);
            self.cpu.p.set_zn(self.cpu.a);
            self.cpu.cycles += 1;
        }

        // ROtate Right
        pub(super) fn ror(&mut self, operand: Operand) {
            let mut data = self.read(operand);
            let c = data.nth(0);

            data >>= 1;
            if self.cpu.p.contains(Status::C) {
                data |= 0x80;
            }
            self.cpu.p.set(Status::C, c == 1);
            self.cpu.p.set_zn(data);
            self.write(operand, data);
            self.cpu.cycles += 1;
        }

        pub(super) fn ror_for_accumelator(&mut self, _: Operand) {
            let c = self.cpu.a.nth(0);

            let mut a = self.cpu.a >> 1;
            if self.cpu.p.contains(Status::C) {
                a |= 0x80;
            }
            self.cpu.a = a;
            self.cpu.p.set(Status::C, c == 1);
            self.cpu.p.set_zn(self.cpu.a);
            self.cpu.cycles += 1;
        }

        // JuMP
        pub(super) fn jmp(&mut self, operand: Operand) {
            self.cpu.pc = operand
        }

        // Jump to SubRoutine
        pub(super) fn jsr(&mut self, operand: Operand) {
            self.push_stack_word(self.cpu.pc - 1);
            self.cpu.cycles += 1;
            self.cpu.pc = operand
        }

        // ReTurn from Subroutine
        pub(super) fn rts(&mut self, _: Operand) {
            self.cpu.cycles += 1;
            self.cpu.cycles += 1;
            self.cpu.cycles += 1;
            self.cpu.pc = self.pull_stack_word() + 1
        }

        // ReTurn from Interrupt
        pub(super) fn rti(&mut self, _: Operand) {
            // https://wiki.selfdev.com/w/index.php/Status_flags#The_B_flag
            // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
            self.cpu.cycles += 1;
            self.cpu.cycles += 1;
            self.cpu.p =
                Status::from_bits_truncate(self.pull_stack().into()) & !Status::B | Status::R;
            self.cpu.pc = self.pull_stack_word()
        }

        // Branch if Carry Clear
        pub(super) fn bcc(&mut self, operand: Operand) {
            if !self.cpu.p.contains(Status::C) {
                self.branch(operand)
            }
        }

        // Branch if Carry Set
        pub(super) fn bcs(&mut self, operand: Operand) {
            if self.cpu.p.contains(Status::C) {
                self.branch(operand)
            }
        }

        // Branch if EQual
        pub(super) fn beq(&mut self, operand: Operand) {
            if self.cpu.p.contains(Status::Z) {
                self.branch(operand)
            }
        }

        // Branch if MInus
        pub(super) fn bmi(&mut self, operand: Operand) {
            if self.cpu.p.contains(Status::N) {
                self.branch(operand)
            }
        }

        // Branch if NotEqual
        pub(super) fn bne(&mut self, operand: Operand) {
            if !self.cpu.p.contains(Status::Z) {
                self.branch(operand)
            }
        }

        // Branch if PLus
        pub(super) fn bpl(&mut self, operand: Operand) {
            if !self.cpu.p.contains(Status::N) {
                self.branch(operand)
            }
        }

        // Branch if oVerflow Clear
        pub(super) fn bvc(&mut self, operand: Operand) {
            if !self.cpu.p.contains(Status::V) {
                self.branch(operand)
            }
        }

        // Branch if oVerflow Set
        pub(super) fn bvs(&mut self, operand: Operand) {
            if self.cpu.p.contains(Status::V) {
                self.branch(operand)
            }
        }

        // CLear Carry
        pub(super) fn clc(&mut self, _: Operand) {
            self.cpu.p.remove(Status::C);
            self.cpu.cycles += 1
        }

        // CLear Decimal
        pub(super) fn cld(&mut self, _: Operand) {
            self.cpu.p.remove(Status::D);
            self.cpu.cycles += 1
        }

        // Clear Interrupt
        pub(super) fn cli(&mut self, _: Operand) {
            self.cpu.p.remove(Status::I);
            self.cpu.cycles += 1
        }

        // CLear oVerflow
        pub(super) fn clv(&mut self, _: Operand) {
            self.cpu.p.remove(Status::V);
            self.cpu.cycles += 1
        }

        // SEt Carry flag
        pub(super) fn sec(&mut self, _: Operand) {
            self.cpu.p.insert(Status::C);
            self.cpu.cycles += 1
        }

        // SEt Decimal flag
        pub(super) fn sed(&mut self, _: Operand) {
            self.cpu.p |= Status::D;
            self.cpu.cycles += 1
        }

        // SEt Interrupt disable
        pub(super) fn sei(&mut self, _: Operand) {
            self.cpu.p.set(Status::I, true);
            self.cpu.cycles += 1
        }

        // BReaK(force interrupt)
        pub(super) fn brk(&mut self, _: Operand) {
            self.push_stack_word(self.cpu.pc);
            // https://wiki.selfdev.com/w/index.php/Status_flags#The_B_flag
            // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
            self.push_stack((self.cpu.p | Status::INTERRUPTED_B).bits().into());
            self.cpu.cycles += 1;
            self.cpu.pc = self.read_word(0xFFFEu16.into());
        }

        // No OPeration
        pub(super) fn nop(&mut self, _: Operand) {
            self.cpu.cycles += 1;
        }

        pub(super) fn branch(&mut self, operand: Operand) {
            self.cpu.cycles += 1;
            let offset = <Word as Into<u16>>::into(operand) as i8;
            if page_crossed(offset, self.cpu.pc) {
                self.cpu.cycles += 1;
            }
            self.cpu.pc += offset as u16
        }

        // Load Accumulator and X register
        pub(super) fn lax(&mut self, operand: Operand) {
            let data = self.read(operand);
            self.cpu.a = data;
            self.cpu.x = data;
            self.cpu.p.set_zn(data);
        }

        // Store Accumulator and X register
        pub(super) fn sax(&mut self, operand: Operand) {
            self.write(operand, self.cpu.a & self.cpu.x)
        }

        // Decrement memory and ComPare to accumulator
        pub(super) fn dcp(&mut self, operand: Operand) {
            let result = self.read(operand) - 1;
            self.cpu.p.set_zn(result);
            self.write(operand, result);

            self.cmp(operand)
        }

        // Increment memory and SuBtract with carry
        pub(super) fn isb(&mut self, operand: Operand) {
            let result = self.read(operand) + 1;
            self.cpu.p.set_zn(result);
            self.write(operand, result);

            self.sbc(operand)
        }

        // arithmetic Shift Left and bitwise Or with accumulator
        pub(super) fn slo(&mut self, operand: Operand) {
            let mut data = self.read(operand);

            self.cpu.p.set(Status::C, data.nth(7) == 1);
            data <<= 1;
            self.cpu.p.set_zn(data);
            self.write(operand, data);

            self.ora(operand)
        }

        // Rotate Left and bitwise And with accumulator
        pub(super) fn rla(&mut self, operand: Operand) {
            // rotateLeft excluding tick
            let mut data = self.read(operand);
            let c = data & 0x80;

            data <<= 1;
            if self.cpu.p.contains(Status::C) {
                data |= 0x01
            }
            self.cpu.p.remove(Status::C | Status::Z | Status::N);
            self.cpu.p.set(Status::C, c.u8() == 0x80);
            self.cpu.p.set_zn(data);

            self.write(operand, data);

            self.and(operand)
        }

        // logical Shift Right and bitwise Exclusive or
        pub(super) fn sre(&mut self, operand: Operand) {
            // logicalShiftRight excluding tick
            let mut data = self.read(operand);

            self.cpu.p.set(Status::C, data.nth(0) == 1);
            data >>= 1;
            self.cpu.p.set_zn(data);
            self.write(operand, data);

            self.eor(operand)
        }

        // Rotate Right and Add with carry
        pub(super) fn rra(&mut self, operand: Operand) {
            // rotateRight excluding tick
            let mut data = self.read(operand);
            let c = data.nth(0);

            data >>= 1;
            if self.cpu.p.contains(Status::C) {
                data |= 0x80
            }
            self.cpu.p.set(Status::C, c == 1);
            self.cpu.p.set_zn(data);

            self.write(operand, data);

            self.adc(operand)
        }
    }
}

impl Status {
    fn set_zn(&mut self, value: impl Into<u16>) {
        let v: u16 = value.into();
        self.set(Self::Z, v == 0);
        self.set(Self::N, (v >> 7) & 1 == 1);
    }
}

impl Nes {
    fn push_stack(&mut self, value: Byte) {
        self.write(Word::from(self.cpu.s) + 0x100, value);
        self.cpu.s -= 1;
    }

    fn push_stack_word(&mut self, word: Word) {
        self.push_stack((word >> 8).byte());
        self.push_stack((word & 0xFF).byte());
    }

    fn pull_stack(&mut self) -> Byte {
        self.cpu.s += 1;
        self.read(Word::from(self.cpu.s) + 0x100)
    }

    fn pull_stack_word(&mut self) -> Word {
        let l: Word = self.pull_stack().into();
        let h: Word = self.pull_stack().into();
        h << 8 | l
    }
}

fn page_crossed_u16(value: impl Into<u16>, from: impl Into<u16>) -> bool {
    let a = value.into();
    let b = from.into();
    (b.wrapping_add(a) & 0xFF00) != (b & 0xFF00)
}

fn page_crossed(value: impl Into<i64>, from: impl Into<i64>) -> bool {
    let a = value.into();
    let b = from.into();
    (b.wrapping_add(a) & 0xFF00) != (b & 0xFF00)
}

mod interrupt {
    use super::*;

    impl Cpu {
        pub(crate) fn interrupted(&self) -> bool {
            self.p.contains(Status::I)
        }
    }

    impl Nes {
        pub(crate) fn reset(&mut self) {
            self.cpu.cycles += 5;
            self.cpu.pc = self.read_word(0xFFFCu16.into());
            self.cpu.p.insert(Status::I);
            self.cpu.s -= 3;
        }

        // NMI
        pub(crate) fn non_markable_interrupt(&mut self) {
            self.cpu.cycles += 2;
            self.push_stack_word(self.cpu.pc);
            // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
            // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
            self.push_stack((self.cpu.p | Status::INTERRUPTED_B).bits().into());
            self.cpu.p.insert(Status::I);
            self.cpu.pc = self.read_word(0xFFFAu16.into())
        }

        // IRQ
        pub(crate) fn interrupt_request(&mut self) {
            self.cpu.cycles += 2;
            self.push_stack_word(self.cpu.pc);
            // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
            // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
            self.push_stack((self.cpu.p | Status::INTERRUPTED_B).bits().into());
            self.cpu.p.insert(Status::I);
            self.cpu.pc = self.read_word(0xFFFEu16.into())
        }

        // BRK
        pub(crate) fn break_interrupt(&mut self) {
            self.cpu.cycles += 2;
            self.cpu.pc += 1;
            self.push_stack_word(self.cpu.pc);
            // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
            // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
            self.push_stack((self.cpu.p | Status::INTERRUPTED_B).bits().into());
            self.cpu.p.insert(Status::I);
            self.cpu.pc = self.read_word(0xFFFEu16.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack() {
        let mut nes = Nes::default();

        nes.cpu.s = 0xFF.into();

        nes.push_stack(0x83.into());
        nes.push_stack(0x14.into());

        assert_eq!(nes.pull_stack(), 0x14.into());
        assert_eq!(nes.pull_stack(), 0x83.into());
    }

    #[test]
    fn test_stack_word() {
        let mut nes = Nes::default();

        nes.cpu.s = 0xFF.into();

        nes.push_stack_word(0x98AFu16.into());
        nes.push_stack_word(0x003Au16.into());

        assert_eq!(nes.pull_stack_word(), 0x003Au16.into());
        assert_eq!(nes.pull_stack_word(), 0x98AFu16.into());
    }
}
