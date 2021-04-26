use super::*;

use crate::bus::{ReadOnIndirect, ReadWord};

type Operand = Word;

pub(super) fn decode(opcode: Byte) -> Instruction {
    let (m, am) = match opcode.u8() {
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
    };
    Instruction {
        mnemonic: m,
        addressing_mode: am,
    }
}

pub(super) fn execute<M: Bus, C: CpuClock>(nes: &mut Nes, instruction: Instruction) {
    use self::instruction_set::*;

    let operand = get_operand::<M, C>(nes, &instruction.addressing_mode);

    match (instruction.mnemonic, instruction.addressing_mode) {
        (Mnemonic::LDA, _) => lda::<M, C>(nes, operand),
        (Mnemonic::LDX, _) => ldx::<M, C>(nes, operand),
        (Mnemonic::LDY, _) => ldy::<M, C>(nes, operand),
        (Mnemonic::STA, AddressingMode::IndirectIndexed) => {
            sta::<M, C>(nes, operand);
            C::tick(nes);
        }
        (Mnemonic::STA, _) => sta::<M, C>(nes, operand),
        (Mnemonic::STX, _) => stx::<M, C>(nes, operand),
        (Mnemonic::STY, _) => sty::<M, C>(nes, operand),
        (Mnemonic::TAX, _) => tax::<M, C>(nes),
        (Mnemonic::TSX, _) => tsx::<M, C>(nes),
        (Mnemonic::TAY, _) => tay::<M, C>(nes),
        (Mnemonic::TXA, _) => txa::<M, C>(nes),
        (Mnemonic::TXS, _) => txs::<M, C>(nes),
        (Mnemonic::TYA, _) => tya::<M, C>(nes),
        (Mnemonic::PHA, _) => pha::<M, C>(nes),
        (Mnemonic::PHP, _) => php::<M, C>(nes),
        (Mnemonic::PLA, _) => pla::<M, C>(nes),
        (Mnemonic::PLP, _) => plp::<M, C>(nes),
        (Mnemonic::AND, _) => and::<M, C>(nes, operand),
        (Mnemonic::EOR, _) => eor::<M, C>(nes, operand),
        (Mnemonic::ORA, _) => ora::<M, C>(nes, operand),
        (Mnemonic::BIT, _) => bit::<M, C>(nes, operand),
        (Mnemonic::ADC, _) => adc::<M, C>(nes, operand),
        (Mnemonic::SBC, _) => sbc::<M, C>(nes, operand),
        (Mnemonic::CMP, _) => cmp::<M, C>(nes, operand),
        (Mnemonic::CPX, _) => cpx::<M, C>(nes, operand),
        (Mnemonic::CPY, _) => cpy::<M, C>(nes, operand),
        (Mnemonic::INC, _) => inc::<M, C>(nes, operand),
        (Mnemonic::INX, _) => inx::<M, C>(nes),
        (Mnemonic::INY, _) => iny::<M, C>(nes),
        (Mnemonic::DEC, _) => dec::<M, C>(nes, operand),
        (Mnemonic::DEX, _) => dex::<M, C>(nes),
        (Mnemonic::DEY, _) => dey::<M, C>(nes),
        (Mnemonic::ASL, AddressingMode::Accumulator) => asl_for_accumelator::<M, C>(nes),
        (Mnemonic::ASL, _) => asl::<M, C>(nes, operand),
        (Mnemonic::LSR, AddressingMode::Accumulator) => lsr_for_accumelator::<M, C>(nes),
        (Mnemonic::LSR, _) => lsr::<M, C>(nes, operand),
        (Mnemonic::ROL, AddressingMode::Accumulator) => rol_for_accumelator::<M, C>(nes),
        (Mnemonic::ROL, _) => rol::<M, C>(nes, operand),
        (Mnemonic::ROR, AddressingMode::Accumulator) => ror_for_accumelator::<M, C>(nes),
        (Mnemonic::ROR, _) => ror::<M, C>(nes, operand),
        (Mnemonic::JMP, _) => jmp::<M, C>(nes, operand),
        (Mnemonic::JSR, _) => jsr::<M, C>(nes, operand),
        (Mnemonic::RTS, _) => rts::<M, C>(nes),
        (Mnemonic::RTI, _) => rti::<M, C>(nes),
        (Mnemonic::BCC, _) => bcc::<M, C>(nes, operand),
        (Mnemonic::BCS, _) => bcs::<M, C>(nes, operand),
        (Mnemonic::BEQ, _) => beq::<M, C>(nes, operand),
        (Mnemonic::BMI, _) => bmi::<M, C>(nes, operand),
        (Mnemonic::BNE, _) => bne::<M, C>(nes, operand),
        (Mnemonic::BPL, _) => bpl::<M, C>(nes, operand),
        (Mnemonic::BVC, _) => bvc::<M, C>(nes, operand),
        (Mnemonic::BVS, _) => bvs::<M, C>(nes, operand),
        (Mnemonic::CLC, _) => clc::<M, C>(nes),
        (Mnemonic::CLD, _) => cld::<M, C>(nes),
        (Mnemonic::CLI, _) => cli::<M, C>(nes),
        (Mnemonic::CLV, _) => clv::<M, C>(nes),
        (Mnemonic::SEC, _) => sec::<M, C>(nes),
        (Mnemonic::SED, _) => sed::<M, C>(nes),
        (Mnemonic::SEI, _) => sei::<M, C>(nes),
        (Mnemonic::BRK, _) => brk::<M, C>(nes),
        (Mnemonic::NOP, _) => nop::<M, C>(nes),
        (Mnemonic::LAX, _) => lax::<M, C>(nes, operand),
        (Mnemonic::SAX, _) => sax::<M, C>(nes, operand),
        (Mnemonic::DCP, _) => dcp::<M, C>(nes, operand),
        (Mnemonic::ISB, _) => isb::<M, C>(nes, operand),
        (Mnemonic::SLO, _) => slo::<M, C>(nes, operand),
        (Mnemonic::RLA, _) => rla::<M, C>(nes, operand),
        (Mnemonic::SRE, _) => sre::<M, C>(nes, operand),
        (Mnemonic::RRA, _) => rra::<M, C>(nes, operand),
    }
}

fn get_operand<M: Bus, C: CpuClock>(nes: &mut Nes, addressing_mode: &AddressingMode) -> Operand {
    match addressing_mode {
        AddressingMode::Implicit => Word::from(0x00u16),
        AddressingMode::Accumulator => nes.cpu.a.into(),
        AddressingMode::Immediate => {
            let operand = nes.cpu.pc;
            nes.cpu.pc += 1;
            operand
        }
        AddressingMode::ZeroPage => {
            let operand = Word::from(M::read(nes.cpu.pc, nes)) & 0xFF;
            nes.cpu.pc += 1;
            operand
        }
        AddressingMode::ZeroPageX => {
            let operand = (Word::from(M::read(nes.cpu.pc, nes)) + Word::from(nes.cpu.x)) & 0xFF;
            nes.cpu.pc += 1;
            C::tick(nes);
            operand
        }
        AddressingMode::ZeroPageY => {
            let operand = (Word::from(M::read(nes.cpu.pc, nes)) + Word::from(nes.cpu.y)) & 0xFF;
            nes.cpu.pc += 1;
            C::tick(nes);
            operand
        }
        AddressingMode::Absolute => {
            let operand = M::read_word(nes.cpu.pc, nes);
            nes.cpu.pc += 2;
            operand
        }
        AddressingMode::AbsoluteX { penalty } => {
            let data = M::read_word(nes.cpu.pc, nes);
            let operand = data + Word::from(nes.cpu.x);
            nes.cpu.pc += 2;
            if *penalty {
                if page_crossed_u16(nes.cpu.x, data) {
                    C::tick(nes);
                }
            } else {
                C::tick(nes);
            }
            operand
        }
        AddressingMode::AbsoluteY { penalty } => {
            let data = M::read_word(nes.cpu.pc, nes);
            let operand = data + Word::from(nes.cpu.y);
            nes.cpu.pc += 2;
            if *penalty {
                if page_crossed_u16(nes.cpu.y, data) {
                    C::tick(nes);
                }
            } else {
                C::tick(nes);
            }
            operand
        }
        AddressingMode::Relative => {
            let operand: Word = M::read(nes.cpu.pc, nes).into();
            nes.cpu.pc += 1;
            operand
        }
        AddressingMode::Indirect => {
            let data = M::read_word(nes.cpu.pc, nes);
            let operand = M::read_on_indirect(data, nes);
            nes.cpu.pc += 2;
            operand
        }
        AddressingMode::IndexedIndirect => {
            let data = M::read(nes.cpu.pc, nes);
            let operand = M::read_on_indirect(Word::from(data + nes.cpu.x) & 0xFF, nes);
            nes.cpu.pc += 1;
            C::tick(nes);
            operand
        }
        AddressingMode::IndirectIndexed => {
            let y: Word = nes.cpu.y.into();
            let data: Word = M::read(nes.cpu.pc, nes).into();
            let operand = M::read_on_indirect(data, nes) + y;
            nes.cpu.pc += 1;
            if page_crossed_u16(y, operand - y) {
                C::tick(nes);
            }
            operand
        }
    }
}

mod instruction_set {
    use super::*;

    // LoaD Accumulator
    pub fn lda<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        nes.cpu.a = M::read(operand, nes);
        nes.cpu.p.set_zn(nes.cpu.a)
    }

    // LoaD X register
    pub fn ldx<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        nes.cpu.x = M::read(operand, nes);
        nes.cpu.p.set_zn(nes.cpu.x)
    }

    // LoaD Y register
    pub fn ldy<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        nes.cpu.y = M::read(operand, nes);
        nes.cpu.p.set_zn(nes.cpu.y)
    }

    // STore Accumulator
    pub fn sta<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        M::write(operand, nes.cpu.a, nes)
    }

    // STore X register
    pub fn stx<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        M::write(operand, nes.cpu.x, nes)
    }

    // STore Y register
    pub fn sty<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        M::write(operand, nes.cpu.y, nes)
    }

    // Transfer Accumulator to X
    pub fn tax<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.x = nes.cpu.a;
        nes.cpu.p.set_zn(nes.cpu.x);
        C::tick(nes);
    }

    // Transfer Stack pointer to X
    pub fn tsx<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.x = nes.cpu.s;
        nes.cpu.p.set_zn(nes.cpu.x);
        C::tick(nes);
    }

    // Transfer Accumulator to Y
    pub fn tay<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.y = nes.cpu.a;
        nes.cpu.p.set_zn(nes.cpu.y);
        C::tick(nes);
    }

    // Transfer X to Accumulator
    pub fn txa<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.a = nes.cpu.x;
        nes.cpu.p.set_zn(nes.cpu.a);
        C::tick(nes);
    }

    // Transfer X to Stack pointer
    pub fn txs<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.s = nes.cpu.x;
        C::tick(nes);
    }

    // Transfer Y to Accumulator
    pub fn tya<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.a = nes.cpu.y;
        nes.cpu.p.set_zn(nes.cpu.a);
        C::tick(nes);
    }

    // PusH Accumulator
    pub fn pha<M: Bus, C: CpuClock>(nes: &mut Nes) {
        push_stack::<M>(nes.cpu.a, nes);
        C::tick(nes);
    }

    // PusH Processor status
    pub fn php<M: Bus, C: CpuClock>(nes: &mut Nes) {
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
        push_stack::<M>((nes.cpu.p | Status::OPERATED_B).bits().into(), nes);
        C::tick(nes);
    }

    // PulL Accumulator
    pub fn pla<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.a = pull_stack::<M>(nes);
        nes.cpu.p.set_zn(nes.cpu.a);
        C::tick(nes);
        C::tick(nes);
    }

    // PulL Processor status
    pub fn plp<M: Bus, C: CpuClock>(nes: &mut Nes) {
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
        nes.cpu.p =
            Status::from_bits_truncate(pull_stack::<M>(nes).into()) & !Status::B | Status::R;
        C::tick(nes);
        C::tick(nes);
    }

    // bitwise AND with accumulator
    pub fn and<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let value = M::read(operand, nes);
        nes.cpu.a &= value;
        nes.cpu.p.set_zn(nes.cpu.a);
    }

    // bitwise Exclusive OR
    pub fn eor<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let value = M::read(operand, nes);
        nes.cpu.a ^= value;
        nes.cpu.p.set_zn(nes.cpu.a);
    }

    // bitwise OR with Accumulator
    pub fn ora<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let value = M::read(operand, nes);
        nes.cpu.a |= value;
        nes.cpu.p.set_zn(nes.cpu.a);
    }

    // test BITs
    pub fn bit<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let value = M::read(operand, nes);
        let data = nes.cpu.a & value;
        nes.cpu.p.set(Status::Z, data.u8() == 0);
        nes.cpu.p.set(Status::V, value.nth(6) == 1);
        nes.cpu.p.set(Status::N, value.nth(7) == 1);
    }

    // ADd with Carry
    pub fn adc<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let a = nes.cpu.a;
        let val = M::read(operand, nes);
        let mut result = a + val;

        if nes.cpu.p.contains(Status::C) {
            result += 1;
        }

        // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
        let a7 = a.nth(7);
        let v7 = val.nth(7);
        let c6 = a7 ^ v7 ^ result.nth(7);
        let c7 = (a7 & v7) | (a7 & c6) | (v7 & c6);

        nes.cpu.p.set(Status::C, c7 == 1);
        nes.cpu.p.set(Status::V, (c6 ^ c7) == 1);

        nes.cpu.a = result;
        nes.cpu.p.set_zn(nes.cpu.a)
    }

    // SuBtract with carry
    pub fn sbc<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let a = nes.cpu.a;
        let val = !M::read(operand, nes);
        let mut result = a + val;

        if nes.cpu.p.contains(Status::C) {
            result += 1;
        }

        // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
        let a7 = a.nth(7);
        let v7 = val.nth(7);
        let c6 = a7 ^ v7 ^ result.nth(7);
        let c7 = (a7 & v7) | (a7 & c6) | (v7 & c6);

        nes.cpu.p.set(Status::C, c7 == 1);
        nes.cpu.p.set(Status::V, (c6 ^ c7) == 1);

        nes.cpu.a = result;
        nes.cpu.p.set_zn(nes.cpu.a)
    }

    // CoMPare accumulator
    pub fn cmp<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let cmp = Word::from(nes.cpu.a) - Word::from(M::read(operand, nes));
        let cmp_i16 = <Word as Into<i16>>::into(cmp);

        nes.cpu.p.set(Status::C, 0 <= cmp_i16);
        nes.cpu.p.set_zn(cmp_i16 as u16);
    }

    // ComPare X register
    pub fn cpx<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let value = M::read(operand, nes);
        let cmp = nes.cpu.x - value;

        nes.cpu.p.set(Status::C, value <= nes.cpu.x);
        nes.cpu.p.set_zn(cmp);
    }

    // ComPare Y register
    pub fn cpy<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let value = M::read(operand, nes);
        let cmp = nes.cpu.y - value;

        nes.cpu.p.set(Status::C, value <= nes.cpu.y);
        nes.cpu.p.set_zn(cmp);
    }

    // INCrement memory
    pub fn inc<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let result = M::read(operand, nes) + 1;

        nes.cpu.p.set_zn(result);
        M::write(operand, result, nes);
        C::tick(nes)
    }

    // INcrement X register
    pub fn inx<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.x += 1;
        nes.cpu.p.set_zn(nes.cpu.x);
        C::tick(nes)
    }

    // INcrement Y register
    pub fn iny<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.y += 1;
        nes.cpu.p.set_zn(nes.cpu.y);
        C::tick(nes)
    }

    // DECrement memory
    pub fn dec<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let result = M::read(operand, nes) - 1;

        nes.cpu.p.set_zn(result);
        M::write(operand, result, nes);
        C::tick(nes)
    }

    // DEcrement X register
    pub fn dex<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.x -= 1;
        nes.cpu.p.set_zn(nes.cpu.x);
        C::tick(nes)
    }

    // DEcrement Y register
    pub fn dey<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.y -= 1;
        nes.cpu.p.set_zn(nes.cpu.y);
        C::tick(nes)
    }

    // Arithmetic Shift Left
    pub fn asl<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let mut data = M::read(operand, nes);

        nes.cpu.p.set(Status::C, data.nth(7) == 1);
        data <<= 1;
        nes.cpu.p.set_zn(data);

        M::write(operand, data, nes);
        C::tick(nes);
    }

    pub fn asl_for_accumelator<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.p.set(Status::C, nes.cpu.a.nth(7) == 1);
        nes.cpu.a <<= 1;
        nes.cpu.p.set_zn(nes.cpu.a);

        C::tick(nes);
    }

    // Logical Shift Right
    pub fn lsr<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let mut data = M::read(operand, nes);

        nes.cpu.p.set(Status::C, data.nth(0) == 1);
        data >>= 1;
        nes.cpu.p.set_zn(data);

        M::write(operand, data, nes);
        C::tick(nes);
    }

    pub fn lsr_for_accumelator<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.p.set(Status::C, nes.cpu.a.nth(0) == 1);
        nes.cpu.a >>= 1;
        nes.cpu.p.set_zn(nes.cpu.a);

        C::tick(nes);
    }

    // ROtate Left
    pub fn rol<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let mut data = M::read(operand, nes);
        let c = data.nth(7);

        data <<= 1;
        if nes.cpu.p.contains(Status::C) {
            data |= 0x01;
        }
        nes.cpu.p.set(Status::C, c == 1);
        nes.cpu.p.set_zn(data);
        M::write(operand, data, nes);
        C::tick(nes);
    }

    pub fn rol_for_accumelator<M: Bus, C: CpuClock>(nes: &mut Nes) {
        let c = nes.cpu.a.nth(7);

        let mut a = nes.cpu.a << 1;
        if nes.cpu.p.contains(Status::C) {
            a |= 0x01;
        }
        nes.cpu.a = a;
        nes.cpu.p.set(Status::C, c == 1);
        nes.cpu.p.set_zn(nes.cpu.a);
        C::tick(nes);
    }

    // ROtate Right
    pub fn ror<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let mut data = M::read(operand, nes);
        let c = data.nth(0);

        data >>= 1;
        if nes.cpu.p.contains(Status::C) {
            data |= 0x80;
        }
        nes.cpu.p.set(Status::C, c == 1);
        nes.cpu.p.set_zn(data);
        M::write(operand, data, nes);
        C::tick(nes);
    }

    pub fn ror_for_accumelator<M: Bus, C: CpuClock>(nes: &mut Nes) {
        let c = nes.cpu.a.nth(0);

        let mut a = nes.cpu.a >> 1;
        if nes.cpu.p.contains(Status::C) {
            a |= 0x80;
        }
        nes.cpu.a = a;
        nes.cpu.p.set(Status::C, c == 1);
        nes.cpu.p.set_zn(nes.cpu.a);
        C::tick(nes);
    }

    // JuMP
    pub fn jmp<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        nes.cpu.pc = operand
    }

    // Jump to SubRoutine
    pub fn jsr<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        push_stack_word::<M>(nes.cpu.pc - 1, nes);
        C::tick(nes);
        nes.cpu.pc = operand
    }

    // ReTurn from Subroutine
    pub fn rts<M: Bus, C: CpuClock>(nes: &mut Nes) {
        C::tick(nes);
        C::tick(nes);
        C::tick(nes);
        nes.cpu.pc = pull_stack_word::<M>(nes) + 1
    }

    // ReTurn from Interrupt
    pub fn rti<M: Bus, C: CpuClock>(nes: &mut Nes) {
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
        C::tick(nes);
        C::tick(nes);
        nes.cpu.p =
            Status::from_bits_truncate(pull_stack::<M>(nes).into()) & !Status::B | Status::R;
        nes.cpu.pc = pull_stack_word::<M>(nes)
    }

    // Branch if Carry Clear
    pub fn bcc<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        if !nes.cpu.p.contains(Status::C) {
            branch::<M, C>(nes, operand)
        }
    }

    // Branch if Carry Set
    pub fn bcs<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        if nes.cpu.p.contains(Status::C) {
            branch::<M, C>(nes, operand)
        }
    }

    // Branch if EQual
    pub fn beq<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        if nes.cpu.p.contains(Status::Z) {
            branch::<M, C>(nes, operand)
        }
    }

    // Branch if MInus
    pub fn bmi<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        if nes.cpu.p.contains(Status::N) {
            branch::<M, C>(nes, operand)
        }
    }

    // Branch if NotEqual
    pub fn bne<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        if !nes.cpu.p.contains(Status::Z) {
            branch::<M, C>(nes, operand)
        }
    }

    // Branch if PLus
    pub fn bpl<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        if !nes.cpu.p.contains(Status::N) {
            branch::<M, C>(nes, operand)
        }
    }

    // Branch if oVerflow Clear
    pub fn bvc<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        if !nes.cpu.p.contains(Status::V) {
            branch::<M, C>(nes, operand)
        }
    }

    // Branch if oVerflow Set
    pub fn bvs<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        if nes.cpu.p.contains(Status::V) {
            branch::<M, C>(nes, operand)
        }
    }

    // CLear Carry
    pub fn clc<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.p.remove(Status::C);
        C::tick(nes)
    }

    // CLear Decimal
    pub fn cld<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.p.remove(Status::D);
        C::tick(nes)
    }

    // Clear Interrupt
    pub fn cli<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.p.remove(Status::I);
        C::tick(nes)
    }

    // CLear oVerflow
    pub fn clv<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.p.remove(Status::V);
        C::tick(nes)
    }

    // SEt Carry flag
    pub fn sec<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.p.insert(Status::C);
        C::tick(nes)
    }

    // SEt Decimal flag
    pub fn sed<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.p |= Status::D;
        C::tick(nes)
    }

    // SEt Interrupt disable
    pub fn sei<M: Bus, C: CpuClock>(nes: &mut Nes) {
        nes.cpu.p.set(Status::I, true);
        C::tick(nes)
    }

    // BReaK(force interrupt)
    pub fn brk<M: Bus, C: CpuClock>(nes: &mut Nes) {
        push_stack_word::<M>(nes.cpu.pc, nes);
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
        push_stack::<M>((nes.cpu.p | Status::INTERRUPTED_B).bits().into(), nes);
        C::tick(nes);
        nes.cpu.pc = M::read_word(0xFFFEu16.into(), nes);
    }

    // No OPeration
    pub fn nop<M: Bus, C: CpuClock>(nes: &mut Nes) {
        C::tick(nes);
    }

    pub fn branch<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        C::tick(nes);
        let offset = <Word as Into<u16>>::into(operand) as i8;
        if page_crossed(offset, nes.cpu.pc) {
            C::tick(nes);
        }
        nes.cpu.pc += offset as u16
    }

    // Load Accumulator and X register
    pub fn lax<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let data = M::read(operand, nes);
        nes.cpu.a = data;
        nes.cpu.x = data;
        nes.cpu.p.set_zn(data);
    }

    // Store Accumulator and X register
    pub fn sax<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        M::write(operand, nes.cpu.a & nes.cpu.x, nes)
    }

    // Decrement memory and ComPare to accumulator
    pub fn dcp<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let result = M::read(operand, nes) - 1;
        nes.cpu.p.set_zn(result);
        M::write(operand, result, nes);

        cmp::<M, C>(nes, operand)
    }

    // Increment memory and SuBtract with carry
    pub fn isb<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let result = M::read(operand, nes) + 1;
        nes.cpu.p.set_zn(result);
        M::write(operand, result, nes);

        sbc::<M, C>(nes, operand)
    }

    // arithmetic Shift Left and bitwise Or with accumulator
    pub fn slo<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        let mut data = M::read(operand, nes);

        nes.cpu.p.set(Status::C, data.nth(7) == 1);
        data <<= 1;
        nes.cpu.p.set_zn(data);
        M::write(operand, data, nes);

        ora::<M, C>(nes, operand)
    }

    // Rotate Left and bitwise And with accumulator
    pub fn rla<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        // rotateLeft excluding tick
        let mut data = M::read(operand, nes);
        let c = data & 0x80;

        data <<= 1;
        if nes.cpu.p.contains(Status::C) {
            data |= 0x01
        }
        nes.cpu.p.remove(Status::C | Status::Z | Status::N);
        nes.cpu.p.set(Status::C, c.u8() == 0x80);
        nes.cpu.p.set_zn(data);

        M::write(operand, data, nes);

        and::<M, C>(nes, operand)
    }

    // logical Shift Right and bitwise Exclusive or
    pub fn sre<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        // logicalShiftRight excluding tick
        let mut data = M::read(operand, nes);

        nes.cpu.p.set(Status::C, data.nth(0) == 1);
        data >>= 1;
        nes.cpu.p.set_zn(data);
        M::write(operand, data, nes);

        eor::<M, C>(nes, operand)
    }

    // Rotate Right and Add with carry
    pub fn rra<M: Bus, C: CpuClock>(nes: &mut Nes, operand: Operand) {
        // rotateRight excluding tick
        let mut data = M::read(operand, nes);
        let c = data.nth(0);

        data >>= 1;
        if nes.cpu.p.contains(Status::C) {
            data |= 0x80
        }
        nes.cpu.p.set(Status::C, c == 1);
        nes.cpu.p.set_zn(data);

        M::write(operand, data, nes);

        adc::<M, C>(nes, operand)
    }

    impl Status {
        fn set_zn(&mut self, value: impl Into<u16>) {
            let v: u16 = value.into();
            self.set(Self::Z, v == 0);
            self.set(Self::N, (v >> 7) & 1 == 1);
        }
    }
}
