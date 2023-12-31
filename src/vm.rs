use crate::bytecode::{Bytecode, SIZE_INDEX, SIZE_INSTRUCTION};
use crate::chunk::Chunk;
use crate::object::{Object, Value};
use crate::symbol_table::SymbolTable;

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
    pub fn new() -> Vm {
        Vm {
            stack: Vec::new(),
            chunk: Chunk::new(),
            ip: 0,
        }
    }

    pub fn interpret(
        &mut self,
        globals: &mut SymbolTable,
        chunk: Chunk,
    ) -> Result<Object, VmError> {
        dbg!(&globals);
        self.load_chunk(chunk)?;
        while self.ip < self.chunk.data.len() {
            dbg!(&self.stack);
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
                Bytecode::None => {
                    self.stack.push(Object::new_none());
                    self.ip += SIZE_INSTRUCTION;
                }
                Bytecode::True => {
                    self.stack.push(Object::new_true());
                    self.ip += SIZE_INSTRUCTION;
                }
                Bytecode::False => {
                    self.stack.push(Object::new_false());
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
                    let object = match globals.get(index) {
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
                    let index = globals.get_index(&lhs.name);
                    lhs.value = rhs.value.clone();
                    globals.set(index, lhs.clone());
                    self.stack.push(lhs);
                    self.ip += SIZE_INSTRUCTION;
                }

                Bytecode::And
                | Bytecode::Or
                | Bytecode::Equal
                | Bytecode::NotEqual
                | Bytecode::Less
                | Bytecode::LessEqual
                | Bytecode::Greater
                | Bytecode::GreaterEqual => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    let result = logic_op(&op, &lhs, &rhs)?;
                    self.stack.push(result);
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

    fn load_chunk(&mut self, chunk: Chunk) -> Result<(), VmError> {
        self.chunk = chunk;
        self.ip = 0;
        self.stack.clear();
        Ok(())
    }
}

fn logic_op(op: &Bytecode, lhs: &Object, rhs: &Object) -> Result<Object, VmError> {
    let result = match op {
        Bytecode::And => Value::new_from_bool(lhs.is_truthy() && rhs.is_truthy()),
        Bytecode::Or => Value::new_from_bool(lhs.is_truthy() || rhs.is_truthy()),
        Bytecode::Equal => Value::new_from_bool(lhs.value == rhs.value),
        Bytecode::NotEqual => Value::new_from_bool(lhs.value != rhs.value),
        Bytecode::Less => Value::new_from_bool(lhs.value < rhs.value),
        Bytecode::LessEqual => Value::new_from_bool(lhs.value <= rhs.value),
        Bytecode::Greater => Value::new_from_bool(lhs.value > rhs.value),
        Bytecode::GreaterEqual => Value::new_from_bool(lhs.value >= rhs.value),
        _ => unreachable!(),
    };
    Ok(Object::new(result))
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
