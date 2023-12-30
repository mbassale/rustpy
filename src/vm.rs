use crate::bytecode::{Bytecode, SIZE_INDEX, SIZE_INSTRUCTION};
use crate::chunk::Chunk;
use crate::object::{Object, Value};

#[derive(Clone, Debug)]
pub enum VmError {
    InvalidBytecode(String),
    InvalidOperand(String),
    UndefinedName(String),
}

pub struct Vm {
    stack: Vec<Object>,
    chunk: Chunk,
    ip: usize,
}

impl Vm {
    pub fn new(chunk: Chunk) -> Vm {
        Vm {
            stack: Vec::new(),
            chunk,
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

                // Globals Manipulation
                Bytecode::GetGlobal => {
                    let index = self.chunk.get_data_u64(self.ip + 1);
                    let object = match self.chunk.globals.get(index) {
                        Some(obj) => obj,
                        None => {
                            return Err(VmError::UndefinedName(format!(
                                "NameError: name '{}' not defined",
                                index
                            )))
                        }
                    };
                    self.stack.push(object.clone());
                    self.ip += SIZE_INSTRUCTION + SIZE_INDEX;
                }
                Bytecode::SetGlobal => {
                    let rhs = self.stack.pop().unwrap();
                    let mut lhs = self.stack.pop().unwrap();
                    let index = self.chunk.globals.get_index(&lhs.name);
                    lhs.value = rhs.value.clone();
                    self.chunk.globals.set(index, lhs.clone());
                    self.stack.push(lhs);
                    self.ip += SIZE_INSTRUCTION;
                }

                // Binary Ops
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
            _ => Object::new_none(),
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

    let apply_i64_op = |lhs: i64, rhs: i64| -> Result<Value, VmError> {
        let result = match op {
            Bytecode::Add => lhs + rhs,
            Bytecode::Sub => lhs - rhs,
            Bytecode::Mul => lhs * rhs,
            Bytecode::Div => lhs / rhs,
            __ => return Err(unsupported_operand_types()),
        };
        Ok(Value::Integer(result))
    };

    let apply_f64_op = |lhs: f64, rhs: f64| -> Result<Value, VmError> {
        let result = match op {
            Bytecode::Add => lhs + rhs,
            Bytecode::Sub => lhs - rhs,
            Bytecode::Mul => lhs * rhs,
            Bytecode::Div => lhs / rhs,
            _ => return Err(unsupported_operand_types()),
        };
        Ok(Value::Float(result))
    };

    let apply_string_op = |lhs: &str, rhs: &str| -> Result<Value, VmError> {
        match op {
            Bytecode::Add => Ok(Value::String(lhs.to_string() + rhs)),
            _ => return Err(unsupported_operand_types()),
        }
    };

    let result = match &lhs.value {
        Value::Integer(lhs_val) => match &rhs.value {
            Value::Integer(rhs_val) => Object::new(apply_i64_op(*lhs_val, *rhs_val)?),
            Value::Float(rhs_val) => Object::new(apply_f64_op(*lhs_val as f64, *rhs_val)?),
            _ => return Err(unsupported_operand_types()),
        },
        Value::Float(lhs_val) => match &rhs.value {
            Value::Integer(rhs_val) => Object::new(apply_f64_op(*lhs_val, *rhs_val as f64)?),
            Value::Float(rhs_val) => Object::new(apply_f64_op(*lhs_val, *rhs_val)?),
            _ => return Err(unsupported_operand_types()),
        },
        Value::String(lhs_val) => match &rhs.value {
            Value::String(rhs_val) => Object::new(apply_string_op(&lhs_val, &rhs_val)?),
            _ => return Err(unsupported_operand_types()),
        },
        _ => return Err(unsupported_operand_types()),
    };
    Ok(result)
}
