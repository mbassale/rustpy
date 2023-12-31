use std::fmt::Debug;

use crate::bytecode::{Bytecode, SIZE_INDEX, SIZE_INSTRUCTION};
use crate::chunk::Chunk;

pub struct Instruction {
    op: Bytecode,
    index: Option<u64>,
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.index {
            Some(index) => f.write_fmt(format_args!("({:?}, {})", self.op, index)),
            None => f.write_fmt(format_args!("({:?})", self.op)),
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
                        op: Bytecode::Unknown,
                        index: None,
                    });
                    continue;
                }
            };
            match op {
                Bytecode::Nop => {
                    result.push(Instruction { op, index: None });
                    ip += SIZE_INSTRUCTION;
                }
                Bytecode::Const => {
                    result.push(Instruction {
                        op,
                        index: Some(self.chunk.get_data_u64(ip + 1)),
                    });
                    ip += SIZE_INSTRUCTION + SIZE_INDEX;
                }
                Bytecode::SetGlobal => {
                    result.push(Instruction { op, index: None });
                    ip += SIZE_INSTRUCTION;
                }
                Bytecode::GetGlobal => {
                    result.push(Instruction {
                        op,
                        index: Some(self.chunk.get_data_u64(ip + 1)),
                    });
                    ip += SIZE_INSTRUCTION + SIZE_INDEX;
                }
                Bytecode::Neg | Bytecode::Add | Bytecode::Sub | Bytecode::Mul | Bytecode::Div => {
                    result.push(Instruction { op, index: None });
                    ip += SIZE_INSTRUCTION;
                }
                _ => {
                    result.push(Instruction {
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
