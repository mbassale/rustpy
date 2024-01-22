use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const SIZE_INSTRUCTION: usize = 1;
pub const SIZE_INDEX: usize = 8;

#[derive(Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Bytecode {
    Nop = 0x00,

    // Literals
    None = 0x01,
    True = 0x02,
    False = 0x03,
    Const = 0x04,

    // Stack Manipulation
    Pop = 0x10,

    // Unary Ops
    Not = 0x11,
    Neg = 0x12,

    // Globals & Locals Manipulation
    SetGlobal = 0x20,
    GetGlobal = 0x21,
    SetLocal = 0x22,
    GetLocal = 0x23,

    // Functions and Objects
    Call = 0x30,
    Return = 0x31,

    // Control Flow
    Jump = 0x40,
    JumpIfFalse = 0x41,
    Loop = 0x42,

    // Binary Ops
    And = 0x50,
    Or = 0x51,
    Equal = 0x52,
    NotEqual = 0x53,
    Less = 0x54,
    LessEqual = 0x55,
    Greater = 0x56,
    GreaterEqual = 0x57,

    Add = 0x59,
    Sub = 0x60,
    Mul = 0x61,
    Div = 0x62,

    // For disassembler usage
    Unknown = 0xFF,
}
