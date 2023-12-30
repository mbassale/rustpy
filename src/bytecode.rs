use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const SIZE_INSTRUCTION: usize = 1;
pub const SIZE_INDEX: usize = 8;

#[derive(Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Bytecode {
    Nop = 0x00,

    // Stack Manipulation
    Const = 0x01,
    Push = 0x02,
    Pop = 0x03,

    // Globals Manipulation
    SetGlobal = 0x04,
    GetGlobal = 0x05,

    // Binary Ops
    Neg = 0x54,
    Add = 0x55,
    Sub = 0x56,
    Mul = 0x57,
    Div = 0x58,
}
