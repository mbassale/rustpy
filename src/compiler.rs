use crate::ast::{
    AssignmentExpression, BinaryExpression, BlockExpression, CallExpression, Expression,
    FunctionExpression, IfExpression, Literal, Operator, Program, ReturnExpression,
    UnaryExpression, WhileExpression,
};
use crate::bytecode::Bytecode;
use crate::chunk::Chunk;
use crate::function::Function;
use crate::object::{Object, Value};
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

    pub fn compile(&mut self) -> Result<Function, CompilerError> {
        self.emit_program()
    }

    fn emit_program(&mut self) -> Result<Function, CompilerError> {
        let mut function = Function::new(String::from("<main>"));
        self.program
            .clone()
            .stmts
            .iter()
            .for_each(|expr: &Box<Expression>| {
                self.emit_expression(&mut function, expr.as_ref());
            });
        // Always finish with a Nop opcode
        function.chunk.emit(Bytecode::Nop);
        Ok(function)
    }

    fn emit_expression(&mut self, function: &mut Function, expr: &Expression) {
        let chunk = &mut function.chunk;
        match expr {
            Expression::Function(function_expression) => {
                let name = function_expression.name.to_string();
                let mut child_function = Function::new(name.to_string());
                self.emit_function_expression(&mut child_function, &function_expression);
                let function_object = Object::new(Value::Function(child_function));
                self.globals.insert(&name, Some(function_object));
            }
            Expression::Call(call_expression) => {
                self.emit_call_expression(function, &call_expression)
            }
            Expression::Block(block_expression) => {
                self.emit_block_expression(function, &block_expression)
            }
            Expression::If(if_expression) => self.emit_if_expression(function, &if_expression),
            Expression::While(while_expression) => {
                self.emit_while_expression(function, &while_expression)
            }
            Expression::Return(return_expression) => {
                self.emit_return_expression(function, &return_expression)
            }
            Expression::Assignment(assignment) => self.emit_assignment_op(function, &assignment),
            Expression::Unary(unary) => self.emit_unary_op(function, &unary),
            Expression::Binary(binary) => self.emit_binary_op(function, &binary),
            Expression::Variable(value) => self.emit_variable_op(function, &value),
            Expression::Literal(identifier) => self.emit_literal(chunk, &identifier),
            Expression::Empty => chunk.emit(Bytecode::Nop),
            _ => (),
        };
    }

    fn emit_function_expression(
        &mut self,
        function: &mut Function,
        function_expression: &FunctionExpression,
    ) {
        self.emit_block_expression(function, &function_expression.body);
    }

    fn emit_call_expression(&mut self, function: &mut Function, call_expression: &CallExpression) {
        call_expression.args.iter().for_each(|expr| {
            self.emit_expression(function, expr);
        });
        self.emit_expression(function, call_expression.callable.as_ref());
        function.chunk.emit(Bytecode::Call);
    }

    fn emit_block_expression(&mut self, function: &mut Function, block_expr: &BlockExpression) {
        block_expr.exprs.iter().for_each(|expr| {
            self.emit_expression(function, expr.as_ref());
        });
    }

    fn emit_if_expression(&mut self, function: &mut Function, if_expr: &IfExpression) {
        // Emit If branch
        let mut exit_jump_addrs: Vec<u64> = Vec::new();

        let exit_jump_addr = self.emit_if_branch(
            function,
            if_expr.condition.as_ref(),
            if_expr.then_branch.as_ref(),
        );
        exit_jump_addrs.push(exit_jump_addr);

        // Emit Elif branches
        if_expr.elif_branches.iter().for_each(|elif_expr| {
            let exit_jump_addr = self.emit_if_branch(
                function,
                elif_expr.condition.as_ref(),
                elif_expr.then_branch.as_ref(),
            );
            exit_jump_addrs.push(exit_jump_addr);
        });

        // Emit Else branch
        self.emit_expression(function, if_expr.else_branch.as_ref());

        // Patch exit addresses to prevent fallthrough
        let next_addr = function.chunk.size();
        exit_jump_addrs.iter().for_each(|addr| {
            function.chunk.patch_jump_addr(*addr, next_addr);
        });
    }

    fn emit_if_branch(
        &mut self,
        function: &mut Function,
        condition: &Expression,
        then_branch: &Expression,
    ) -> u64 {
        self.emit_expression(function, condition);
        function.chunk.emit(Bytecode::JumpIfFalse);
        let jump_offset_addr = function.chunk.emit_index(0);
        self.emit_expression(function, then_branch);
        function.chunk.emit(Bytecode::Jump);
        let exit_offset_addr = function.chunk.emit_index(0);
        function
            .chunk
            .patch_jump_addr(jump_offset_addr, function.chunk.size());
        exit_offset_addr
    }

    fn emit_while_expression(&mut self, function: &mut Function, while_expr: &WhileExpression) {
        // emit conditional
        let start_addr = function.chunk.size();
        self.emit_expression(function, while_expr.condition.as_ref());
        function.chunk.emit(Bytecode::JumpIfFalse);
        let jump_offset_addr = function.chunk.emit_index(0);

        // emit body
        self.emit_expression(function, while_expr.body.as_ref());

        // loop to the beginning
        let chunk = &mut function.chunk;
        chunk.emit(Bytecode::Loop);
        chunk.emit_index(start_addr);

        // exit address
        let exit_addr = chunk.size();
        chunk.patch_jump_addr(jump_offset_addr, exit_addr);
    }

    fn emit_return_expression(
        &mut self,
        function: &mut Function,
        return_expression: &ReturnExpression,
    ) {
        match return_expression.expr.as_ref() {
            Expression::Empty => function.chunk.emit(Bytecode::None),
            _ => self.emit_expression(function, return_expression.expr.as_ref()),
        };
        function.chunk.emit(Bytecode::Return);
    }

    fn emit_assignment_op(
        &mut self,
        function: &mut Function,
        assignment_expr: &AssignmentExpression,
    ) {
        self.emit_expression(function, assignment_expr.lhs.as_ref());
        self.emit_expression(function, assignment_expr.rhs.as_ref());
        function.chunk.emit(Bytecode::SetGlobal);
    }

    fn emit_unary_op(&mut self, function: &mut Function, unary_expr: &UnaryExpression) {
        self.emit_expression(function, unary_expr.expr.as_ref());
        self.emit_op(&mut function.chunk, &unary_expr.op);
    }

    fn emit_binary_op(&mut self, function: &mut Function, binary_expr: &BinaryExpression) {
        self.emit_expression(function, binary_expr.lhs.as_ref());
        self.emit_expression(function, binary_expr.rhs.as_ref());
        self.emit_op(&mut function.chunk, &binary_expr.op);
    }

    fn emit_variable_op(&mut self, function: &mut Function, identifier: &String) {
        let index: u64;
        if self.globals.contains_name(identifier) {
            index = self.globals.get_index(identifier);
        } else {
            index = self.globals.insert(identifier, None);
        }
        function.chunk.emit(Bytecode::GetGlobal);
        function.chunk.emit_index(index);
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
