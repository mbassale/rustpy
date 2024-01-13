use crate::bytecode::{Bytecode, SIZE_INDEX, SIZE_INSTRUCTION};
use crate::chunk::Chunk;
use crate::function::Function;
use crate::object::{Object, Value};
use crate::symbol_table::SymbolTable;

#[derive(Clone, Debug)]
pub enum VmError {
    InvalidBytecode(String),
    InvalidOperand(String),
    UndefinedName(String),
}

pub struct Frame {
    function: Function,
    stack_size: usize,
    ip: usize,
}

impl Frame {
    fn get_chunk(&self) -> &Chunk {
        &self.function.chunk
    }

    fn get_opcode(&self) -> Result<Bytecode, VmError> {
        let op = self.function.chunk.data[self.ip];
        let op = match Bytecode::try_from(op) {
            Ok(op) => op,
            Err(_) => {
                return Err(VmError::InvalidBytecode(format!(
                    "Invalid bytecode: {}",
                    op
                )))
            }
        };
        Ok(op)
    }

    fn set_ip(&mut self, addr: usize) {
        self.ip = addr;
    }

    fn incr_ip(&mut self, offset: usize) {
        self.ip += offset;
    }
}

pub struct Vm {
    stack: Vec<Object>,
    frames: Vec<Frame>,
}

impl Vm {
    pub fn new() -> Vm {
        Vm {
            stack: Vec::new(),
            frames: Vec::new(),
        }
    }

    fn current_frame(&mut self) -> &mut Frame {
        match self.frames.last_mut() {
            Some(frame) => frame,
            _ => unreachable!(),
        }
    }

    fn dump_stack(&self) {
        dbg!(&self.stack);
    }

    pub fn interpret(
        &mut self,
        globals: &mut SymbolTable,
        function: Function,
    ) -> Result<Object, VmError> {
        dbg!(&globals);
        self.stack.clear();
        self.frames.push(Frame {
            function,
            stack_size: 0,
            ip: 0,
        });

        while !self.frames.is_empty() {
            let ret_val = self.interpret_function(globals)?;
            self.stack.push(ret_val);
        }

        let result = match self.stack.pop() {
            Some(value) => value,
            _ => Object::new_none(),
        };
        Ok(result)
    }

    fn interpret_function(&mut self, globals: &mut SymbolTable) -> Result<Object, VmError> {
        while self.current_frame().ip < self.current_frame().get_chunk().data.len() {
            let op = self.current_frame().get_opcode()?;
            println!("IP: {:X} OpCode: {:?}", self.current_frame().ip, op);
            self.dump_stack();

            match op {
                Bytecode::Nop => {
                    self.current_frame().incr_ip(SIZE_INSTRUCTION);
                }

                // Literals
                Bytecode::None => {
                    self.stack.push(Object::new_none());
                    self.current_frame().incr_ip(SIZE_INSTRUCTION);
                }
                Bytecode::True => {
                    self.stack.push(Object::new_true());
                    self.current_frame().incr_ip(SIZE_INSTRUCTION);
                }
                Bytecode::False => {
                    self.stack.push(Object::new_false());
                    self.current_frame().incr_ip(SIZE_INSTRUCTION);
                }
                Bytecode::Const => {
                    let offset_addr = self.current_frame().ip + SIZE_INSTRUCTION;
                    let index = self.current_frame().get_chunk().get_data_u64(offset_addr);
                    let constant = &self.current_frame().get_chunk().constants[index as usize];
                    let object = Object::from_literal(&constant);
                    self.stack.push(object);
                    self.current_frame().incr_ip(SIZE_INSTRUCTION + SIZE_INDEX);
                }
                Bytecode::Pop => {
                    self.stack.pop().unwrap();
                    self.current_frame().incr_ip(SIZE_INSTRUCTION);
                }

                // Globals Manipulation
                Bytecode::GetGlobal => {
                    let index_addr = self.current_frame().ip + SIZE_INSTRUCTION;
                    let index = self.current_frame().get_chunk().get_data_u64(index_addr);
                    let global_obj = match globals.get(index) {
                        Some(obj) => obj,
                        None => {
                            return Err(VmError::UndefinedName(format!(
                                "NameError: name '{}' not defined",
                                index
                            )))
                        }
                    };
                    self.stack.push(global_obj.clone());
                    self.current_frame().incr_ip(SIZE_INSTRUCTION + SIZE_INDEX);
                }
                Bytecode::SetGlobal => {
                    let index_addr = self.current_frame().ip + SIZE_INSTRUCTION;
                    let index = self.current_frame().get_chunk().get_data_u64(index_addr);
                    let rhs = self.stack.pop().unwrap();
                    let global_obj = globals.get_mut(index);
                    global_obj.value = rhs.value;
                    self.current_frame().incr_ip(SIZE_INSTRUCTION + SIZE_INDEX);
                }

                // Locals Manipulation
                Bytecode::GetLocal => {
                    let index_addr = self.current_frame().ip + SIZE_INSTRUCTION;
                    let stack_offset = self.current_frame().get_chunk().get_data_u64(index_addr);
                    let local_obj = self.stack[stack_offset as usize].clone();
                    self.stack.push(local_obj);
                    self.current_frame().incr_ip(SIZE_INSTRUCTION + SIZE_INDEX);
                }
                Bytecode::SetLocal => {
                    let index_addr = self.current_frame().ip + SIZE_INSTRUCTION;
                    let stack_offset = self.current_frame().get_chunk().get_data_u64(index_addr);
                    match self.stack.last() {
                        Some(local_obj) => {
                            self.stack[stack_offset as usize] = local_obj.clone();
                        }
                        None => panic!("SetLocal on empty stack"),
                    };
                    self.current_frame().incr_ip(SIZE_INSTRUCTION + SIZE_INDEX);
                }

                Bytecode::Call => {
                    let callable = self.stack.pop().unwrap();
                    let index = globals.get_index(&callable.name);
                    let function_obj = match globals.get(index) {
                        Some(obj) => obj,
                        None => {
                            return Err(VmError::UndefinedName(format!(
                                "NameError: name '{}' not defined",
                                index
                            )))
                        }
                    };
                    match &function_obj.value {
                        Value::Function(function) => {
                            self.current_frame().incr_ip(SIZE_INSTRUCTION);
                            self.frames.push(Frame {
                                function: function.clone(),
                                stack_size: self.stack.len(),
                                ip: 0,
                            });
                        }
                        _ => {
                            return Err(VmError::InvalidOperand(format!(
                                "Invalid callable: '{}'",
                                &callable.name
                            )));
                        }
                    }
                }

                Bytecode::Return => {
                    let ret_val = self.stack.pop().unwrap();
                    let stack_size = self.current_frame().stack_size;
                    assert!(self.frames.pop().is_some());
                    // pop frame locals
                    println!(
                        "Stack Size: {}, New Stack Size: {}",
                        self.stack.len(),
                        stack_size
                    );
                    self.stack.resize(stack_size, Object::new_none());
                    return Ok(ret_val);
                }

                // Control Flow
                Bytecode::Jump => {
                    let offset_addr = self.current_frame().ip + SIZE_INSTRUCTION;
                    let addr_offset = self.current_frame().get_chunk().get_data_u64(offset_addr);
                    println!(
                        "{:?} IP: {:X}, AddrOffset: {:X}, Result: {:X}",
                        op,
                        self.current_frame().ip,
                        addr_offset,
                        offset_addr
                    );
                    self.current_frame().incr_ip(addr_offset as usize);
                }

                Bytecode::JumpIfFalse => {
                    // we remove the conditional value from the stack
                    let conditional_value = self.stack.pop().unwrap();
                    if conditional_value.is_falsey() {
                        let offset_addr = self.current_frame().ip + SIZE_INSTRUCTION;
                        let addr_offset =
                            self.current_frame().get_chunk().get_data_u64(offset_addr);
                        println!(
                            "{:?} IP: {:X}, AddrOffset: {:X}, Result: {:X}",
                            op,
                            self.current_frame().ip,
                            addr_offset,
                            offset_addr,
                        );
                        self.current_frame().incr_ip(addr_offset as usize);
                    } else {
                        self.current_frame().incr_ip(SIZE_INSTRUCTION + SIZE_INDEX);
                    }
                }

                Bytecode::Loop => {
                    let addr_addr = self.current_frame().ip + SIZE_INSTRUCTION;
                    let addr = self.current_frame().get_chunk().get_data_u64(addr_addr);
                    println!(
                        "{:?} IP: {:X}, Addr: {:X}",
                        op,
                        self.current_frame().ip,
                        addr,
                    );
                    self.current_frame().set_ip(addr as usize);
                }

                // Unary Ops
                Bytecode::Not => {
                    let rhs = self.stack.pop().unwrap();
                    let result = Object::new(Value::new_from_bool(rhs.is_falsey()));
                    self.stack.push(result);
                    self.current_frame().incr_ip(SIZE_INSTRUCTION);
                }
                Bytecode::Neg => {
                    let rhs = self.stack.pop().unwrap();
                    let result = match rhs.value {
                        Value::Integer(value) => Value::Integer(-value),
                        Value::Float(value) => Value::Float(-value),
                        _ => {
                            return Err(VmError::InvalidOperand(format!(
                                "TypeError: unsupported operand type for '-': {:?}",
                                rhs.value
                            )));
                        }
                    };
                    self.stack.push(Object::new(result));
                    self.current_frame().incr_ip(SIZE_INSTRUCTION);
                }

                // Binary Ops
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
                    self.current_frame().incr_ip(SIZE_INSTRUCTION);
                }

                // Binary Ops
                Bytecode::Add | Bytecode::Sub | Bytecode::Mul | Bytecode::Div => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    let result = binary_op(&op, &lhs, &rhs)?;
                    self.stack.push(result);
                    self.current_frame().incr_ip(SIZE_INSTRUCTION);
                }
                _ => unimplemented!(),
            };
        }

        let stack_size = self.current_frame().stack_size;
        assert!(self.frames.pop().is_some());
        let result = match self.stack.pop() {
            Some(value) => value,
            _ => Object::new_none(),
        };
        self.stack.resize(stack_size, Object::new_none());
        Ok(result)
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
