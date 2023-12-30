use crate::ast::{
    AssignmentExpression, BinaryExpression, Expression, Literal, Operator, Program, UnaryExpression,
};
use crate::bytecode::Bytecode;
use crate::chunk::Chunk;

#[derive(Clone, Debug)]
pub enum CompilerError {}

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
            Expression::Assignment(assignment) => self.emit_assignment_op(chunk, &assignment),
            Expression::Unary(unary) => self.emit_unary_op(chunk, &unary),
            Expression::Binary(binary) => self.emit_binary_op(chunk, &binary),
            Expression::Variable(value) => self.emit_variable_op(chunk, &value),
            Expression::Literal(identifier) => self.emit_literal(chunk, &identifier),
            _ => (),
        };
    }

    fn emit_assignment_op(&self, chunk: &mut Chunk, assignment_expr: &AssignmentExpression) {
        self.emit_expression(chunk, assignment_expr.lhs.as_ref());
        self.emit_expression(chunk, assignment_expr.rhs.as_ref());
        chunk.emit(Bytecode::SetGlobal);
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

    fn emit_variable_op(&self, chunk: &mut Chunk, identifier: &str) {
        let index = chunk.add_global(identifier);
        chunk.emit(Bytecode::GetGlobal);
        chunk.emit_index(index);
    }

    fn emit_op(&self, chunk: &mut Chunk, op: &Operator) {
        match op {
            Operator::Neg => chunk.emit(Bytecode::Neg),
            Operator::Add => chunk.emit(Bytecode::Add),
            Operator::Sub => chunk.emit(Bytecode::Sub),
            Operator::Mul => chunk.emit(Bytecode::Mul),
            Operator::Div => chunk.emit(Bytecode::Div),
            _ => unimplemented!(),
        }
    }

    fn emit_literal(&self, chunk: &mut Chunk, literal: &Literal) {
        let index = chunk.add_constant(literal);
        chunk.emit(Bytecode::Const);
        chunk.emit_index(index as u64);
    }
}
