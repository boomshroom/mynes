#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub addr_mode: AddressMode,
    pub op_code: Opcode,
}

#[derive(Debug, Clone, Copy)]
pub enum AddressMode {
    Implicit,
    Manual,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX(Fix),
    AbsoluteY(Fix),
    Indirect,
    IndexedIndirect,
    IndirectIndexed(Fix),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fix {
    Always,
    Conditional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    ORA,
    AND,
    EOR,
    ADC,
    SBC,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    INC,
    INX,
    INY,
    ASL,
    ROL,
    LSR,
    ROR,
    LDA,
    STA,
    LDX,
    STX,
    LDY,
    STY,
    TAX,
    TXA,
    TAY,
    TYA,
    TSX,
    TXS,
    PLA,
    PHA,
    PLP,
    PHP,
    BPL,
    BMI,
    BVC,
    BVS,
    BCC,
    BCS,
    BNE,
    BEQ,
    BRK,
    RTI,
    JSR,
    RTS,
    JMP,
    BIT,
    CLC,
    SEC,
    CLD,
    SED,
    CLI,
    SEI,
    CLV,
    NOP,

    // Unofficial
    LAX,
    LAS,
    SAX,
    XAA,
    AHX,
    TAS,
    ISB,
    AXS,
    DCP,
    SLO,
    ANC,
    RLA,
    SRE,
    ALR,
    RRA,
    ARR,
    SXA,
    SYA,
    NOPConsume,
    Unofficial(u8),
}
use Opcode::*;

#[inline]
fn int_to_bits(i: u8) -> [bool; 8] {
    [
        i & 1 != 0,
        (i >> 1) & 1 != 0,
        (i >> 2) & 1 != 0,
        (i >> 3) & 1 != 0,
        (i >> 4) & 1 != 0,
        (i >> 5) & 1 != 0,
        (i >> 6) & 1 != 0,
        (i >> 7) & 1 != 0,
    ]
}

#[derive(PartialEq, Eq)]
enum Block {
    Control = 0,
    Alu = 1,
    Rwm = 2,
    Unofficial = 3,
}

#[derive(PartialEq, Eq)]
enum I3 {
    U0 = 0,
    U1 = 1,
    U2 = 2,
    U3 = 3,
    U4 = 4,
    U5 = 5,
    U6 = 6,
    U7 = 7,
}

impl Block {
    fn from_bits(low: bool, high: bool) -> Self {
        match (high, low) {
            (false, false) => Self::Control,
            (false, true) => Self::Alu,
            (true, false) => Self::Rwm,
            (true, true) => Self::Unofficial,
        }
    }
}

impl I3 {
    fn from_bits(high: bool, mid: bool, low: bool) -> Self {
        const T: bool = true;
        const F: bool = false;

        match [high, mid, low] {
            [F, F, F] => I3::U0,
            [F, F, T] => I3::U1,
            [F, T, F] => I3::U2,
            [F, T, T] => I3::U3,
            [T, F, F] => I3::U4,
            [T, F, T] => I3::U5,
            [T, T, F] => I3::U6,
            [T, T, T] => I3::U7,
        }
    }
}

impl Instruction {
    pub fn decode(op: u8) -> Instruction {
        use AddressMode::*;

        let bits = int_to_bits(op);
        let row = I3::from_bits(bits[7], bits[6], bits[5]);
        let col = I3::from_bits(bits[4], bits[3], bits[2]);
        let block = Block::from_bits(bits[0], bits[1]);

        let addr_mode = match block {
            Block::Alu => match col {
                I3::U0 => IndexedIndirect,
                I3::U1 => ZeroPage,
                I3::U2 => Immediate,
                I3::U3 => Absolute,
                I3::U4 if row == I3::U4 => IndirectIndexed(Fix::Always),
                I3::U4 => IndirectIndexed(Fix::Conditional),
                I3::U5 => ZeroPageX,
                I3::U6 if row == I3::U4 => AbsoluteY(Fix::Always),
                I3::U6 => AbsoluteY(Fix::Conditional),
                I3::U7 if row == I3::U4 => AbsoluteX(Fix::Always),
                I3::U7 => AbsoluteX(Fix::Conditional),
            },
            Block::Rwm => match col {
                I3::U0 => match row {
                    I3::U0 | I3::U1 | I3::U2 | I3::U3 => Implicit,
                    I3::U4 | I3::U5 | I3::U6 | I3::U7 => Immediate,
                },
                I3::U1 => ZeroPage,
                I3::U3 => Absolute,
                I3::U5 if row == I3::U4 || row == I3::U5 => ZeroPageY,
                I3::U5 => ZeroPageX,
                I3::U7 => match row {
                    I3::U4 => Manual,
                    I3::U5 => AbsoluteY(Fix::Conditional),
                    _ => AbsoluteX(Fix::Always),
                },
                _ => Implicit,
            },
            Block::Control => match col {
                I3::U0 if row == I3::U1 => Manual,   // JSR
                I3::U3 if row == I3::U3 => Indirect, // JMP
                I3::U0 => match row {
                    I3::U0 | I3::U1 | I3::U2 | I3::U3 => Implicit,
                    I3::U4 | I3::U5 | I3::U6 | I3::U7 => Immediate,
                },
                I3::U1 => ZeroPage,
                I3::U3 => Absolute,
                I3::U4 => Manual,
                I3::U5 => ZeroPageX,
                I3::U7 if row == I3::U4 => Manual,
                I3::U7 => AbsoluteX(Fix::Conditional),
                _ => Implicit,
            },
            Block::Unofficial => match col {
                I3::U0 => IndexedIndirect,
                I3::U1 => ZeroPage,
                I3::U2 => Immediate,
                I3::U3 => Absolute,
                I3::U4 if row == I3::U5 => IndirectIndexed(Fix::Conditional),
                I3::U4 => IndirectIndexed(Fix::Always),
                I3::U5 if row == I3::U4 || row == I3::U5 => ZeroPageY,
                I3::U5 => ZeroPageX,
                I3::U6 if row == I3::U5 => AbsoluteY(Fix::Conditional),
                I3::U6 => AbsoluteY(Fix::Always),
                I3::U7 => match row {
                    I3::U4 => AbsoluteY(Fix::Always),
                    I3::U5 => AbsoluteY(Fix::Conditional),
                    _ => AbsoluteX(Fix::Always),
                },
            },
        };

        let op_code = match block {
            Block::Alu => match row {
                I3::U0 => ORA,
                I3::U1 => AND,
                I3::U2 => EOR,
                I3::U3 => ADC,
                I3::U4 if col == I3::U2 => NOPConsume,
                I3::U4 => STA,
                I3::U5 => LDA,
                I3::U6 => CMP,
                I3::U7 => SBC,
            },
            Block::Control => Instruction::ctrl_instr(col, row),
            Block::Rwm => Instruction::data_instr(op, col, row),
            Block::Unofficial => Instruction::unofficial_instr(col, row),
        };

        Instruction { addr_mode, op_code }
    }

    #[inline]
    fn ctrl_instr(col: I3, row: I3) -> Opcode {
        match (col, row) {
            (I3::U3, I3::U2) | (I3::U3, I3::U3) => JMP,
            (I3::U1, I3::U1) | (I3::U3, I3::U1) => BIT,

            (I3::U0, I3::U0) => BRK,
            (I3::U2, I3::U0) => PHP,
            (I3::U4, I3::U0) => BPL,
            (I3::U6, I3::U0) => CLC,

            (I3::U0, I3::U1) => JSR,
            (I3::U2, I3::U1) => PLP,
            (I3::U4, I3::U1) => BMI,
            (I3::U6, I3::U1) => SEC,

            (I3::U0, I3::U2) => RTI,
            (I3::U2, I3::U2) => PHA,
            (I3::U4, I3::U2) => BVC,
            (I3::U6, I3::U2) => CLI,

            (I3::U0, I3::U3) => RTS,
            (I3::U2, I3::U3) => PLA,
            (I3::U4, I3::U3) => BVS,
            (I3::U6, I3::U3) => SEI,

            (I3::U1, I3::U4) => STY,
            (I3::U2, I3::U4) => DEY,
            (I3::U3, I3::U4) => STY,
            (I3::U4, I3::U4) => BCC,
            (I3::U5, I3::U4) => STY,
            (I3::U6, I3::U4) => TYA,

            (I3::U2, I3::U5) => TAY,
            (I3::U4, I3::U5) => BCS,
            (I3::U6, I3::U5) => CLV,
            (_, I3::U5) => LDY,

            (I3::U0, I3::U6) => CPY,
            (I3::U1, I3::U6) => CPY,
            (I3::U2, I3::U6) => INY,
            (I3::U3, I3::U6) => CPY,
            (I3::U4, I3::U6) => BNE,
            (I3::U6, I3::U6) => CLD,

            (I3::U0, I3::U7) => CPX,
            (I3::U1, I3::U7) => CPX,
            (I3::U2, I3::U7) => INX,
            (I3::U3, I3::U7) => CPX,
            (I3::U4, I3::U7) => BEQ,
            (I3::U6, I3::U7) => SED,

            (I3::U7, I3::U4) => SYA,

            _ => NOPConsume,
        }
    }

    #[inline]
    fn data_instr(op: u8, col: I3, row: I3) -> Opcode {
        use Opcode::*;

        match (col, row) {
            (I3::U4, _) => Unofficial(op),

            (I3::U2, I3::U5) => TAX,
            (I3::U6, I3::U5) => TSX,
            (_, I3::U5) => LDX,

            (I3::U0, I3::U4) => NOPConsume,
            (I3::U0, I3::U6) => NOPConsume,
            (I3::U0, I3::U7) => NOPConsume,
            (I3::U2, I3::U4) => TXA,
            (I3::U6, I3::U4) => TXS,
            (I3::U7, I3::U4) => SXA,
            (_, I3::U4) => STX,

            (I3::U0, _) => Unofficial(op),
            (I3::U6, _) => NOP,

            (I3::U2, I3::U6) => DEX,
            (I3::U2, I3::U7) => NOP,

            (_, I3::U0) => ASL,
            (_, I3::U1) => ROL,
            (_, I3::U2) => LSR,
            (_, I3::U3) => ROR,
            (_, I3::U6) => DEC,
            (_, I3::U7) => INC,
        }
    }

    #[inline]
    fn unofficial_instr(col: I3, row: I3) -> Opcode {
        use Opcode::*;

        match (row, col) {
            (I3::U0, I3::U2) => ANC,
            (I3::U0, _) => SLO,

            (I3::U1, I3::U2) => ANC,
            (I3::U1, _) => RLA,

            (I3::U2, I3::U2) => ALR,
            (I3::U2, _) => SRE,

            (I3::U3, I3::U2) => ARR,
            (I3::U3, _) => RRA,

            (I3::U4, I3::U4) | (I3::U4, I3::U7) => AHX,
            (I3::U4, I3::U6) => TAS,
            (I3::U4, I3::U2) => XAA,
            (I3::U4, _) => SAX,

            (I3::U5, I3::U6) => LAS,
            (I3::U5, _) => LAX,

            (I3::U6, I3::U2) => AXS,
            (I3::U6, _) => DCP,

            (I3::U7, I3::U2) => SBC,
            (I3::U7, _) => ISB,
        }
    }
}
