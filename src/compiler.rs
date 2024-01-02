use crate::ast::{
    AssignmentExpression, BinaryExpression, BlockExpression, Expression, IfExpression, Literal,
    Operator, Program, UnaryExpression, WhileExpression,
};
use crate::bytecode::Bytecode;
use crate::chunk::Chunk;
use crate::symbol_table::SymbolTable;

#[derive(Clone, Debug)]
pub enum CompilerError {}

pub struct Compiler<'a> {
    program: Program,
    globals: &'a mut SymbolTable,
}

impl Compiler<'_> {
    pub fn new(program: Program, globals: &mut SymbolTable) -> Compiler {
        Compiler { program, globals }
    }

    pub fn compile(&mut self) -> Result<Chunk, CompilerError> {
        self.emit_program()
    }

    fn emit_program(&mut self) -> Result<Chunk, CompilerError> {
        let mut chunk = Chunk::new();
        self.program
            .clone()
            .stmts
            .iter()
            .for_each(|expr: &Box<Expression>| {
                self.emit_expression(&mut chunk, expr.as_ref());
            });
        // Always finish with a Nop opcode
        chunk.emit(Bytecode::Nop);
        Ok(chunk)
    }

    fn emit_expression(&mut self, chunk: &mut Chunk, expr: &Expression) {
        match expr {
            Expression::Block(block_expression) => {
                self.emit_block_expression(chunk, &block_expression)
            }
            Expression::If(if_expression) => self.emit_if_expression(chunk, &if_expression),
            Expression::While(while_expression) => {
                self.emit_while_expression(chunk, &while_expression)
            }
            Expression::Assignment(assignment) => self.emit_assignment_op(chunk, &assignment),
            Expression::Unary(unary) => self.emit_unary_op(chunk, &unary),
            Expression::Binary(binary) => self.emit_binary_op(chunk, &binary),
            Expression::Variable(value) => self.emit_variable_op(chunk, &value),
            Expression::Literal(identifier) => self.emit_literal(chunk, &identifier),
            Expression::Empty => chunk.emit(Bytecode::Nop),
            _ => (),
        };
    }

    fn emit_block_expression(&mut self, chunk: &mut Chunk, block_expr: &BlockExpression) {
        block_expr.exprs.iter().for_each(|expr| {
            self.emit_expression(chunk, expr.as_ref());
        });
    }

    fn emit_if_expression(&mut self, chunk: &mut Chunk, if_expr: &IfExpression) {
        // Emit If branch
        let mut exit_jump_addrs: Vec<u64> = Vec::new();

        let exit_jump_addr = self.emit_if_branch(
            chunk,
            if_expr.condition.as_ref(),
            if_expr.then_branch.as_ref(),
        );
        exit_jump_addrs.push(exit_jump_addr);

        // Emit Elif branches
        if_expr.elif_branches.iter().for_each(|elif_expr| {
            let exit_jump_addr = self.emit_if_branch(
                chunk,
                elif_expr.condition.as_ref(),
                elif_expr.then_branch.as_ref(),
            );
            exit_jump_addrs.push(exit_jump_addr);
        });

        // Emit Else branch
        self.emit_expression(chunk, if_expr.else_branch.as_ref());

        // Patch exit addresses to prevent fallthrough
        let next_addr = chunk.size();
        exit_jump_addrs.iter().for_each(|addr| {
            chunk.patch_jump_addr(*addr, next_addr);
        });
    }

    fn emit_if_branch(
        &mut self,
        chunk: &mut Chunk,
        condition: &Expression,
        then_branch: &Expression,
    ) -> u64 {
        self.emit_expression(chunk, condition);
        chunk.emit(Bytecode::JumpIfFalse);
        let jump_offset_addr = chunk.emit_index(0);
        self.emit_expression(chunk, then_branch);
        chunk.emit(Bytecode::Jump);
        let exit_offset_addr = chunk.emit_index(0);
        chunk.patch_jump_addr(jump_offset_addr, chunk.size());
        exit_offset_addr
    }

    fn emit_while_expression(&mut self, chunk: &mut Chunk, while_expr: &WhileExpression) {
        // emit conditional
        let start_addr = chunk.size();
        self.emit_expression(chunk, while_expr.condition.as_ref());
        chunk.emit(Bytecode::JumpIfFalse);
        let jump_offset_addr = chunk.emit_index(0);

        // emit body
        self.emit_expression(chunk, while_expr.body.as_ref());

        // loop to the beginning
        chunk.emit(Bytecode::Loop);
        chunk.emit_index(start_addr);

        // exit address
        let exit_addr = chunk.size();
        chunk.patch_jump_addr(jump_offset_addr, exit_addr);
    }

    fn emit_assignment_op(&mut self, chunk: &mut Chunk, assignment_expr: &AssignmentExpression) {
        self.emit_expression(chunk, assignment_expr.lhs.as_ref());
        self.emit_expression(chunk, assignment_expr.rhs.as_ref());
        chunk.emit(Bytecode::SetGlobal);
    }

    fn emit_unary_op(&mut self, chunk: &mut Chunk, unary_expr: &UnaryExpression) {
        self.emit_expression(chunk, unary_expr.expr.as_ref());
        self.emit_op(chunk, &unary_expr.op);
    }

    fn emit_binary_op(&mut self, chunk: &mut Chunk, binary_expr: &BinaryExpression) {
        self.emit_expression(chunk, binary_expr.lhs.as_ref());
        self.emit_expression(chunk, binary_expr.rhs.as_ref());
        self.emit_op(chunk, &binary_expr.op);
    }

    fn emit_variable_op(&mut self, chunk: &mut Chunk, identifier: &str) {
        let index: u64;
        if self.globals.contains_name(identifier) {
            index = self.globals.get_index(identifier);
        } else {
            index = self.globals.insert(identifier, None);
        }
        chunk.emit(Bytecode::GetGlobal);
        chunk.emit_index(index);
    }

    fn emit_op(&self, chunk: &mut Chunk, op: &Operator) {
        match op {
            Operator::Not => chunk.emit(Bytecode::Not),
            Operator::Neg => chunk.emit(Bytecode::Neg),
            Operator::And => chunk.emit(Bytecode::And),
            Operator::Or => chunk.emit(Bytecode::Or),
            Operator::Equal => chunk.emit(Bytecode::Equal),
            Operator::NotEqual => chunk.emit(Bytecode::NotEqual),
            Operator::Less => chunk.emit(Bytecode::Less),
            Operator::LessEqual => chunk.emit(Bytecode::LessEqual),
            Operator::Greater => chunk.emit(Bytecode::Greater),
            Operator::GreaterEqual => chunk.emit(Bytecode::GreaterEqual),
            Operator::Add => chunk.emit(Bytecode::Add),
            Operator::Sub => chunk.emit(Bytecode::Sub),
            Operator::Mul => chunk.emit(Bytecode::Mul),
            Operator::Div => chunk.emit(Bytecode::Div),
            _ => unimplemented!(),
        }
    }

    fn emit_literal(&self, chunk: &mut Chunk, literal: &Literal) {
        match literal {
            Literal::None => chunk.emit(Bytecode::None),
            Literal::True => chunk.emit(Bytecode::True),
            Literal::False => chunk.emit(Bytecode::False),
            _ => {
                let index = chunk.add_constant(literal);
                chunk.emit(Bytecode::Const);
                chunk.emit_index(index as u64);
            }
        };
    }
}
