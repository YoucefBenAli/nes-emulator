use std::{collections::HashMap, ops::Add};

use lazy_static::lazy_static;
use strum_macros::{AsRefStr, Display};


use crate::AddressingModes::AddressingMode;
use Mnemonic::*;

#[derive(Clone, Copy, Debug, AsRefStr, Display, PartialEq)]
pub enum Mnemonic {
    BRK,
    TAX,
    LDA,
    LDX,
    LDY,
    STA,
    STX,
    STY,
    ADC,
    AND,
    ASL,
    BIT,
    BMI,
    BNE,
    BPL,
    BVC,
    BVS,
    BCC,
    BCS,
    BEQ,
    CLC,
    CLD,
    CLI,
    CLV,
    SEC,
    SED,
    SEI,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    // Illegal Opcodes
    ANC,
    SAX,
    ARR,
    ALR,
    LXA,
    AXA,
    AXS,
    DCP,
    ISB,
    HLT,
    LAS,
    LAX,
    RLA,
    RRA,
    ASO,
    LSE,
    XAS,
    SAY,
    SKW,
    XAA,
    TAS,
}

pub struct OpCode {
    hex_code: u8,
    instruction: Mnemonic,
    num_byte: u8,
    cycles: u8,
    mode: AddressingMode,
}

impl OpCode {
    pub fn new(hex_code: u8, instruction: Mnemonic, num_byte: u8, cycles: u8, mode: AddressingMode) -> Self {
        OpCode{hex_code, instruction, num_byte, cycles, mode}
    }

    pub fn create_hash_map(op_codes: &Vec<OpCode>) -> HashMap<u8, &OpCode>{
        let mut map: HashMap<u8, &OpCode> = HashMap::new();
        for op_code in op_codes {
            map.insert(op_code.hex_code, op_code);
        }

        map
    }

    pub fn get_mode(&self) -> &AddressingMode {
        &self.mode
    }

    pub fn get_num_bytes(&self) -> u8 {
        self.num_byte
    }

    pub fn get_instruction(&self) -> Mnemonic {
        self.instruction
    }
}

lazy_static! {
    pub static ref CPU_OPS_CODES: Vec<OpCode> = vec![
        OpCode::new(0x00, BRK, 1, 7, AddressingMode::NoneAddressing),

        OpCode::new(0xaa, TAX, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0xa8, TAY, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0xa9, LDA, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa5, LDA, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb5, LDA, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xad, LDA, 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbd, LDA, 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        OpCode::new(0xb9, LDA, 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        OpCode::new(0xa1, LDA, 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xb1, LDA, 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

        OpCode::new(0xa2, LDX, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa6, LDX, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb6, LDX, 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0xae, LDX, 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbe, LDX, 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),

        OpCode::new(0xa0, LDY, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa4, LDY, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb4, LDY, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xac, LDY, 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbc, LDY, 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),

        OpCode::new(0x85, STA, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x95, STA, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x8d, STA, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x9d, STA, 3, 5, AddressingMode::Absolute_X),
        OpCode::new(0x99, STA, 3, 5, AddressingMode::Absolute_Y),
        OpCode::new(0x81, STA, 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x91, STA, 2, 6, AddressingMode::Indirect_Y),

        OpCode::new(0x86, STX, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x96, STX, 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0x8e, STX, 3, 4, AddressingMode::Absolute),

        OpCode::new(0x84, STY, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x94, STY, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x8C, STY, 3, 4, AddressingMode::Absolute),

        OpCode::new(0x69, ADC, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x65, ADC, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x75, ADC, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x6d, ADC, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x7d, ADC, 3, 4 /*+1 if page crossed*/, AddressingMode::Absolute_X),
        OpCode::new(0x79, ADC, 3, 4 /*+1 if page crossed*/, AddressingMode::Absolute_Y),
        OpCode::new(0x61, ADC, 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x71, ADC, 2, 5 /*+1 if page crossed*/, AddressingMode::Indirect_Y),

        OpCode::new(0x29, AND, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x25, AND, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x35, AND, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x2d, AND, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x3d, AND, 3, 4 /*+1 if page crossed*/, AddressingMode::Absolute_X),
        OpCode::new(0x39, AND, 3, 4 /*+1 if page crossed*/, AddressingMode::Absolute_Y),
        OpCode::new(0x21, AND, 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x31, AND, 2, 5 /*+1 if page crossed*/, AddressingMode::Indirect_Y),

        OpCode::new(0x0a, ASL, 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x06, ASL, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x16, ASL, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x0e, ASL, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x1e, ASL, 3, 7, AddressingMode::Absolute_X),

        OpCode::new(0x24, BIT, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x2C, BIT, 3, 3, AddressingMode::Absolute),

        OpCode::new(0x30, BMI, 2, 2 /*(+1 if branch succeeds +2 if to a new page) */, AddressingMode::Relative),
        OpCode::new(0xD0, BNE, 2, 2 /*(+1 if branch succeeds +2 if to a new page) */, AddressingMode::Relative),
        OpCode::new(0x10, BPL, 2, 2 /*(+1 if branch succeeds +2 if to a new page) */, AddressingMode::Relative),
        OpCode::new(0x50, BVC, 2, 2 /*(+1 if branch succeeds +2 if to a new page) */, AddressingMode::Relative),
        OpCode::new(0x70, BVS, 2, 2 /*(+1 if branch succeeds +2 if to a new page) */, AddressingMode::Relative),
        OpCode::new(0x90, BCC, 2, 2 /*(+1 if branch succeeds +2 if to a new page) */, AddressingMode::Relative),
        OpCode::new(0xB0, BCS, 2, 2 /*(+1 if branch succeeds +2 if to a new page) */, AddressingMode::Relative),
        OpCode::new(0xF0, BEQ, 2, 2 /*(+1 if branch succeeds +2 if to a new page) */, AddressingMode::Relative),

        OpCode::new(0x18, CLC, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xD8, CLD, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x58, CLI, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xB8, CLV, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x38, SEC, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xF8, SED, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x78, SEI, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0xC9, CMP, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xC5, CMP, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xD5, CMP, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xCD, CMP, 3, 4, AddressingMode::Absolute),
        OpCode::new(0xDD, CMP, 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_X),
        OpCode::new(0xD9, CMP, 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_Y),
        OpCode::new(0xC1, CMP, 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xD1, CMP, 2, 5 /* +1 if page crossed */, AddressingMode::Indirect_Y),

        OpCode::new(0xE0, CPX, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xE4, CPX, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xEC, CPX, 3, 4, AddressingMode::Absolute),

        OpCode::new(0xC0, CPY, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xC4, CPY, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xCC, CPY, 3, 4, AddressingMode::Absolute),

        OpCode::new(0xC6, DEC, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xD6, DEC, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xCE, DEC, 3, 6, AddressingMode::Absolute),
        OpCode::new(0xDE, DEC, 3, 7, AddressingMode::Absolute_X),

        OpCode::new(0xCA, DEX, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x88, DEY, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x49, EOR, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x45, EOR, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x55, EOR, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x4D, EOR, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x5D, EOR, 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_X),
        OpCode::new(0x59, EOR, 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_Y),
        OpCode::new(0x41, EOR, 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x51, EOR, 2, 5 /* +1 if page crossed */, AddressingMode::Indirect_Y),

        OpCode::new(0x09, ORA, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x05, ORA, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x15, ORA, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x0D, ORA, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x1D, ORA, 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_X),
        OpCode::new(0x19, ORA, 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_Y),
        OpCode::new(0x01, ORA, 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x11, ORA, 2, 5 /* +1 if page crossed */, AddressingMode::Indirect_Y),

        OpCode::new(0xE6, INC, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xF6, INC, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xEE, INC, 3, 6, AddressingMode::Absolute),
        OpCode::new(0xFE, INC, 3, 7, AddressingMode::Absolute_X),

        OpCode::new(0xE8, INX, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0xC8, INY, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x4C, JMP, 3, 3, AddressingMode::Absolute),
        OpCode::new(0x6C, JMP, 3, 5, AddressingMode::Indirect),

        OpCode::new(0x20, JSR, 3, 6, AddressingMode::Absolute),

        OpCode::new(0x4A, LSR, 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x46, LSR, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x56, LSR, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x4E, LSR, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x5E, LSR, 3, 7, AddressingMode::Absolute_X),

        OpCode::new(0xEA, NOP, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x48, PHA, 1, 3, AddressingMode::NoneAddressing),

        OpCode::new(0x08, PHP, 1, 3, AddressingMode::NoneAddressing),

        OpCode::new(0x68, PLA, 1, 4, AddressingMode::NoneAddressing),

        OpCode::new(0x28, PLP, 1, 4, AddressingMode::NoneAddressing),

        OpCode::new(0x2A, ROL, 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x26, ROL, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x36, ROL, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x2E, ROL, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x3E, ROL, 3, 7, AddressingMode::Absolute_X),

        OpCode::new(0x6A, ROR, 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x66, ROR, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x76, ROR, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x6E, ROR, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x7E, ROR, 3, 7, AddressingMode::Absolute_X),

        OpCode::new(0x40, RTI, 1, 6, AddressingMode::NoneAddressing),

        OpCode::new(0x60, RTS, 1, 6, AddressingMode::NoneAddressing),

        OpCode::new(0xE9, SBC, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xE5, SBC, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xF5, SBC, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xED, SBC, 3, 4, AddressingMode::Absolute),
        OpCode::new(0xFD, SBC, 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_X),
        OpCode::new(0xF9, SBC, 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_Y),
        OpCode::new(0xE1, SBC, 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xF1, SBC, 2, 5 /* +1 if page crossed */, AddressingMode::Indirect_Y),

        OpCode::new(0xBA, TSX, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x8A, TXA, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x9A, TXS, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x98, TYA, 1, 2, AddressingMode::NoneAddressing),

        // Undocumented opcodes, source: https://www.nesdev.org/undocumented_opcodes.txt
        OpCode::new(0x0B, ANC, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x2B, ANC, 2, 2, AddressingMode::Immediate),

        OpCode::new(0x87, SAX, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x97, SAX, 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0x83, SAX, 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x8F, SAX, 3, 4, AddressingMode::Absolute),

        OpCode::new(0x6B, ARR, 2, 2, AddressingMode::Immediate),

        OpCode::new(0x4B, ALR, 2, 2, AddressingMode::Immediate),

        OpCode::new(0xAB, LXA, 2, 2, AddressingMode::Immediate),

        OpCode::new(0x9F, AXA, 3, 5, AddressingMode::Absolute_Y),
        OpCode::new(0x93, AXA, 2, 6, AddressingMode::Indirect_Y),

        OpCode::new(0xCB, AXS, 2, 2, AddressingMode::Immediate),

        OpCode::new(0xC7, DCP, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xD7, DCP, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xCF, DCP, 3, 6, AddressingMode::Absolute),
        OpCode::new(0xDF, DCP, 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0xDB, DCP, 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0xC3, DCP, 2, 8, AddressingMode::Indirect_X),
        OpCode::new(0xD3, DCP, 2, 8, AddressingMode::Indirect_Y),

        OpCode::new(0x04, NOP, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x14, NOP, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x34, NOP, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x44, NOP, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x54, NOP, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x64, NOP, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x74, NOP, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x80, NOP, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x82, NOP, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x89, NOP, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xC2, NOP, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xD4, NOP, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xE2, NOP, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xF4, NOP, 2, 4, AddressingMode::ZeroPage_X),

        OpCode::new(0xE7, ISB, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xF7, ISB, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xEF, ISB, 3, 6, AddressingMode::Absolute),
        OpCode::new(0xFF, ISB, 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0xFB, ISB, 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0xE3, ISB, 2, 8, AddressingMode::Indirect_X),
        OpCode::new(0xF3, ISB, 2, 8, AddressingMode::Indirect_Y),

        OpCode::new(0x02, HLT, 1, 0, AddressingMode::NoneAddressing),
        OpCode::new(0x12, HLT, 1, 0, AddressingMode::NoneAddressing),
        OpCode::new(0x22, HLT, 1, 0, AddressingMode::NoneAddressing),
        OpCode::new(0x32, HLT, 1, 0, AddressingMode::NoneAddressing),
        OpCode::new(0x42, HLT, 1, 0, AddressingMode::NoneAddressing),
        OpCode::new(0x52, HLT, 1, 0, AddressingMode::NoneAddressing),
        OpCode::new(0x62, HLT, 1, 0, AddressingMode::NoneAddressing),
        OpCode::new(0x72, HLT, 1, 0, AddressingMode::NoneAddressing),
        OpCode::new(0x92, HLT, 1, 0, AddressingMode::NoneAddressing),
        OpCode::new(0xB2, HLT, 1, 0, AddressingMode::NoneAddressing),
        OpCode::new(0xD2, HLT, 1, 0, AddressingMode::NoneAddressing),
        OpCode::new(0xF2, HLT, 1, 0, AddressingMode::NoneAddressing),

        OpCode::new(0xBB, LAS, 3, 4, AddressingMode::Absolute_Y),

        OpCode::new(0xA7, LAX, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xB7, LAX, 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0xAF, LAX, 3, 4, AddressingMode::Absolute),
        OpCode::new(0xBF, LAX, 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0xA3, LAX, 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xB3, LAX, 2, 5, AddressingMode::Indirect_Y),

        OpCode::new(0x1A, NOP, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x3A, NOP, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x5A, NOP, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x7A, NOP, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xDA, NOP, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xFA, NOP, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x27, RLA, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x37, RLA, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x2F, RLA, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x3F, RLA, 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x3B, RLA, 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0x23, RLA, 2, 8, AddressingMode::Indirect_X),
        OpCode::new(0x33, RLA, 2, 8, AddressingMode::Indirect_Y),

        OpCode::new(0x67, RRA, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x77, RRA, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x6F, RRA, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x7F, RRA, 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x7B, RRA, 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0x63, RRA, 2, 8, AddressingMode::Indirect_X),
        OpCode::new(0x73, RRA, 2, 8, AddressingMode::Indirect_Y),

        OpCode::new(0xEB, SBC, 2, 2, AddressingMode::Immediate),

        OpCode::new(0x07, ASO, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x17, ASO, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x0F, ASO, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x1F, ASO, 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x1B, ASO, 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0x03, ASO, 2, 8, AddressingMode::Indirect_X),
        OpCode::new(0x13, ASO, 2, 8, AddressingMode::Indirect_Y),

        OpCode::new(0x47, LSE, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x57, LSE, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x4F, LSE, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x5F, LSE, 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x5B, LSE, 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0x43, LSE, 2, 8, AddressingMode::Indirect_X),
        OpCode::new(0x53, LSE, 2, 8, AddressingMode::Indirect_Y),

        OpCode::new(0x9E, XAS, 3, 5, AddressingMode::Absolute_Y),
        OpCode::new(0x9C, SAY, 3, 5, AddressingMode::Absolute_X),

        OpCode::new(0x0C, SKW, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x1C, SKW, 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x3C, SKW, 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x5C, SKW, 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x7C, SKW, 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0xDC, SKW, 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0xFC, SKW, 3, 4, AddressingMode::Absolute_X),

        OpCode::new(0x8B, XAA, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x9B, TAS, 3, 5, AddressingMode::Absolute_Y),
    ];

    pub static ref OPCODES_MAP: HashMap<u8, &'static OpCode> = OpCode::create_hash_map(&CPU_OPS_CODES);


}
