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

    // Globals Manipulation
    SetGlobal = 0x20,
    GetGlobal = 0x21,

    // Binary Ops
    Neg = 0x54,
    Add = 0x55,
    Sub = 0x56,
    Mul = 0x57,
    Div = 0x58,

    // For disassembler usage
    Unknown = 0xFF,
}
