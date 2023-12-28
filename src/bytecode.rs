#[derive(Clone, Debug)]
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
