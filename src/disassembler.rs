use std::fmt::Debug;

use crate::bytecode::{Bytecode, SIZE_INDEX, SIZE_INSTRUCTION};
use crate::chunk::Chunk;

pub struct Instruction {
    ip: usize,
    op: Bytecode,
    index: Option<u64>,
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.index {
            Some(index) => f.write_fmt(format_args!(
                "{:08X}: {:?}, dec: {} hex: Ox{:X}",
                self.ip, self.op, index, index
            )),
            None => f.write_fmt(format_args!("{:08X}: {:?}", self.ip, self.op)),
        }
    }
}

pub struct Disassembler {
    chunk: Chunk,
}

impl Disassembler {
    pub fn new(chunk: Chunk) -> Disassembler {
        Disassembler { chunk }
    }

    pub fn disassemble(&self) -> Vec<Instruction> {
        let mut result: Vec<Instruction> = Vec::new();
        let mut ip = 0;
        while ip < self.chunk.data.len() {
            let op = self.chunk.data[ip];
            let op = match Bytecode::try_from(op) {
                Ok(op) => op,
                Err(_) => {
                    result.push(Instruction {
                        ip,
                        op: Bytecode::Unknown,
                        index: None,
                    });
                    continue;
                }
            };
            match op {
                Bytecode::Nop | Bytecode::None | Bytecode::True | Bytecode::False => {
                    result.push(Instruction {
                        ip,
                        op,
                        index: None,
                    });
                    ip += SIZE_INSTRUCTION;
                }

                Bytecode::Const => {
                    result.push(Instruction {
                        ip,
                        op,
                        index: self.chunk.get_data_u64_safe(ip + 1),
                    });
                    ip += SIZE_INSTRUCTION + SIZE_INDEX;
                }
                Bytecode::SetGlobal => {
                    result.push(Instruction {
                        ip,
                        op,
                        index: None,
                    });
                    ip += SIZE_INSTRUCTION;
                }
                Bytecode::GetGlobal => {
                    result.push(Instruction {
                        ip,
                        op,
                        index: self.chunk.get_data_u64_safe(ip + 1),
                    });
                    ip += SIZE_INSTRUCTION + SIZE_INDEX;
                }

                Bytecode::Jump | Bytecode::JumpIfFalse | Bytecode::Loop => {
                    result.push(Instruction {
                        ip,
                        op,
                        index: self.chunk.get_data_u64_safe(ip + 1),
                    });
                    ip += SIZE_INSTRUCTION + SIZE_INDEX;
                }

                Bytecode::Call
                | Bytecode::Return
                | Bytecode::Not
                | Bytecode::Neg
                | Bytecode::And
                | Bytecode::Or
                | Bytecode::Equal
                | Bytecode::NotEqual
                | Bytecode::Less
                | Bytecode::LessEqual
                | Bytecode::Greater
                | Bytecode::GreaterEqual
                | Bytecode::Add
                | Bytecode::Sub
                | Bytecode::Mul
                | Bytecode::Div => {
                    result.push(Instruction {
                        ip,
                        op,
                        index: None,
                    });
                    ip += SIZE_INSTRUCTION;
                }
                _ => {
                    result.push(Instruction {
                        ip,
                        op: Bytecode::Unknown,
                        index: None,
                    });
                    ip += SIZE_INSTRUCTION;
                }
            };
        }
        result
    }
}
