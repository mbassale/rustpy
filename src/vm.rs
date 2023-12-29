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
                Bytecode::Add | Bytecode::Sub | Bytecode::Mul | Bytecode::Div => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    let result = binary_op(&op, &lhs, &rhs)?;
                    self.stack.push(result);
                    self.ip += SIZE_INSTRUCTION;
                }
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

fn binary_op(op: &Bytecode, lhs: &Object, rhs: &Object) -> Result<Object, VmError> {
    let unsupported_operand_types = || -> VmError {
        VmError::InvalidOperand(format!(
            "TypeError: unsupported operand type(s) for {:?}: {:?} and {:?}",
            op, rhs, lhs
        ))
    };

    let apply_i64_op = |lhs: i64, rhs: i64| -> Result<i64, VmError> {
        let result = match op {
            Bytecode::Add => lhs + rhs,
            Bytecode::Sub => lhs - rhs,
            Bytecode::Mul => lhs * rhs,
            Bytecode::Div => lhs / rhs,
            __ => return Err(unsupported_operand_types()),
        };
        Ok(result)
    };

    let apply_f64_op = |lhs: f64, rhs: f64| -> Result<f64, VmError> {
        let result = match op {
            Bytecode::Add => lhs + rhs,
            Bytecode::Sub => lhs - rhs,
            Bytecode::Mul => lhs * rhs,
            Bytecode::Div => lhs / rhs,
            _ => return Err(unsupported_operand_types()),
        };
        Ok(result)
    };

    let apply_string_op = |lhs: &str, rhs: &str| -> Result<String, VmError> {
        match op {
            Bytecode::Add => Ok(lhs.to_string() + rhs),
            _ => return Err(unsupported_operand_types()),
        }
    };

    let result = match lhs {
        Object::Integer(lhs_val) => match rhs {
            Object::Integer(rhs_val) => Object::Integer(apply_i64_op(*lhs_val, *rhs_val)?),
            Object::Float(rhs_val) => Object::Float(apply_f64_op(*lhs_val as f64, *rhs_val)?),
            _ => return Err(unsupported_operand_types()),
        },
        Object::Float(lhs_val) => match rhs {
            Object::Integer(rhs_val) => Object::Float(apply_f64_op(*lhs_val, *rhs_val as f64)?),
            Object::Float(rhs_val) => Object::Float(apply_f64_op(*lhs_val, *rhs_val)?),
            _ => return Err(unsupported_operand_types()),
        },
        Object::String(lhs_val) => match rhs {
            Object::String(rhs_val) => Object::String(apply_string_op(lhs_val, rhs_val)?),
            _ => return Err(unsupported_operand_types()),
        },
        _ => return Err(unsupported_operand_types()),
    };
    Ok(result)
}
