use crate::ast::{BinaryExpression, Expression, Operator, Program, UnaryExpression, Value};
use crate::bytecode::Bytecode;

#[derive(Clone, Debug)]
pub enum CompilerError {}

#[derive(Clone, Debug)]
pub struct Chunk {
    pub name: String,
    pub data: Vec<u8>,
    pub constants: Vec<Value>,
}

impl Chunk {
    fn new() -> Chunk {
        Chunk {
            name: String::from("__main__"),
            data: Vec::new(),
            constants: Vec::new(),
        }
    }

    fn add_constant(&mut self, value: &Value) -> usize {
        self.constants.push(value.clone());
        self.constants.len() - 1
    }

    fn emit(&mut self, op: Bytecode) {
        self.data.push(op as u8);
    }

    fn emit_index(&mut self, index: u64) {
        self.data.extend_from_slice(&index.to_ne_bytes());
    }
}

pub struct Compiler {
    program: Program,
}

impl Compiler {
    pub fn new(program: Program) -> Compiler {
        Compiler { program }
    }

    pub fn compile(&mut self) -> Result<Chunk, CompilerError> {
        self.emit_program()
    }

    fn emit_program(&mut self) -> Result<Chunk, CompilerError> {
        let mut chunk = Chunk::new();
        self.program
            .stmts
            .iter()
            .for_each(|expr: &Box<Expression>| {
                self.emit_expression(&mut chunk, expr.as_ref());
            });
        Ok(chunk)
    }

    fn emit_expression(&self, chunk: &mut Chunk, expr: &Expression) {
        match expr {
            Expression::Unary(unary) => self.emit_unary_op(chunk, &unary),
            Expression::Binary(binary) => self.emit_binary_op(chunk, &binary),
            Expression::Literal(value) => self.emit_literal(chunk, &value),
            _ => (),
        };
    }

    fn emit_unary_op(&self, chunk: &mut Chunk, unary_expr: &UnaryExpression) {
        self.emit_expression(chunk, unary_expr.expr.as_ref());
        self.emit_op(chunk, &unary_expr.op);
    }

    fn emit_binary_op(&self, chunk: &mut Chunk, binary_expr: &BinaryExpression) {
        self.emit_expression(chunk, binary_expr.lhs.as_ref());
        self.emit_expression(chunk, binary_expr.rhs.as_ref());
        self.emit_op(chunk, &binary_expr.op);
    }

    fn emit_op(&self, chunk: &mut Chunk, op: &Operator) {
        match op {
            Operator::Neg => chunk.emit(Bytecode::Neg),
            Operator::Add => chunk.emit(Bytecode::Add),
            Operator::Sub => chunk.emit(Bytecode::Sub),
            Operator::Mul => chunk.emit(Bytecode::Mul),
            Operator::Div => chunk.emit(Bytecode::Div),
            _ => (),
        }
    }

    fn emit_literal(&self, chunk: &mut Chunk, value: &Value) {
        let index = chunk.add_constant(value);
        chunk.emit(Bytecode::Const);
        chunk.emit_index(index as u64);
    }
}
