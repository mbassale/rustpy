use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const SIZE_INSTRUCTION: usize = 1;
pub const SIZE_INDEX: usize = 8;

#[derive(Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Bytecode {
    Nop = 0x00,
    Const = 0x01,
    Push = 0x02,
    Pop = 0x03,
    Neg = 0x04,
    Add = 0x05,
    Sub = 0x06,
    Mul = 0x07,
    Div = 0x08,
}
