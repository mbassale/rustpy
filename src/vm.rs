use crate::bytecode::{Bytecode, SIZE_INDEX, SIZE_INSTRUCTION};
use crate::compiler::Chunk;
use crate::object::Object;

#[derive(Clone, Debug)]
pub enum VmError {
    InvalidBytecode(String),
    InvalidOperand(String),
}

pub struct Vm {
    pub stack: Vec<Object>,
    pub chunk: Chunk,
    pub top: usize,
    pub ip: usize,
}

impl Vm {
    pub fn new(chunk: Chunk) -> Vm {
        Vm {
            stack: Vec::new(),
            chunk,
            top: 0,
            ip: 0,
        }
    }

    pub fn interpret(&mut self) -> Result<Object, VmError> {
        while self.ip < self.chunk.data.len() {
            let op = self.chunk.data[self.ip];
            let op = match Bytecode::try_from(op) {
                Ok(op) => op,
                Err(_) => {
                    return Err(VmError::InvalidBytecode(format!(
                        "Invalid bytecode: {}",
                        op
                    )))
                }
            };

            match op {
                Bytecode::Nop => {
                    self.ip += SIZE_INSTRUCTION;
                }
                Bytecode::Const => {
                    let index = self.chunk.get_data_u64(self.ip + 1);
                    let constant = &self.chunk.constants[index as usize];
                    let object = Object::from_literal(&constant);
                    self.stack.push(object);
                    self.ip += SIZE_INSTRUCTION + SIZE_INDEX;
                }
                Bytecode::Add => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();

                    let result = match lhs {
                        Object::Integer(lhs_val) => match rhs {
                            Object::Integer(rhs_val) => Object::Integer(lhs_val + rhs_val),
                            Object::Float(rhs_val) => Object::Float(lhs_val as f64 + rhs_val),
                            _ => {
                                return Err(VmError::InvalidOperand(format!(
                                    "TypeError: unsupported operand type(s) for +: 'int' and {:?}",
                                    rhs
                                )));
                            }
                        },
                        Object::Float(lhs_val) => match rhs {
                            Object::Integer(rhs_val) => Object::Float(lhs_val + rhs_val as f64),
                            Object::Float(rhs_val) => Object::Float(lhs_val + rhs_val),
                            _ => {
                                return Err(VmError::InvalidOperand(format!(
                                "TypeError: unsupported operand type(s) for +: 'float' and {:?}", rhs)
                            ));
                            }
                        },
                        Object::String(lhs_val) => {
                            match rhs {
                                Object::String(rhs_val) => Object::String(lhs_val + &rhs_val),
                                _ => {
                                    return Err(VmError::InvalidOperand(format!("TypeError: unsupported operand type(s) for +: 'str' and {:?}", rhs)));
                                }
                            }
                        }
                        _ => {
                            return Err(VmError::InvalidOperand(format!(
                                "Invalid operands for add: {:?} & {:?}",
                                rhs, lhs
                            )))
                        }
                    };
                    self.stack.push(result);
                    self.ip += SIZE_INSTRUCTION;
                }
                Bytecode::Sub => {}
                Bytecode::Mul => {}
                Bytecode::Div => {}
                _ => unimplemented!(),
            };
        }

        let result = match self.stack.pop() {
            Some(value) => value,
            _ => Object::None,
        };
        Ok(result)
    }
}
