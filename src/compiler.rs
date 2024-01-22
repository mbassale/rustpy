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
pub enum CompilerError {
    NameNotFound(String),
    InvalidExpression(String),
}

struct Local {
    name: String,
    depth: usize,
}

pub struct Compiler<'a> {
    program: Program,
    globals: &'a mut SymbolTable,
    locals: Vec<Local>,
    scope_depth: usize,
    continue_addr_stack: Vec<u64>,
    break_addr_stack: Vec<u64>,
}

impl Compiler<'_> {
    pub fn new(program: Program, globals: &mut SymbolTable) -> Compiler {
        Compiler {
            program,
            globals,
            locals: Vec::new(),
            scope_depth: 0,
            continue_addr_stack: Vec::new(),
            break_addr_stack: Vec::new(),
        }
    }

    fn init_compiler(&mut self) {
        self.locals = Vec::new();
        self.scope_depth = 0;
    }

    pub fn compile(&mut self) -> Result<Function, CompilerError> {
        self.emit_program()
    }

    fn emit_program(&mut self) -> Result<Function, CompilerError> {
        let mut function = Function::new_global_scope();
        self.init_compiler();
        for expr in self.program.stmts.clone() {
            self.emit_expression(&mut function, expr.as_ref())?;
        }
        // Always finish with a Nop opcode
        function.chunk.emit(Bytecode::Nop);
        Ok(function)
    }

    fn emit_expression(
        &mut self,
        function: &mut Function,
        expr: &Expression,
    ) -> Result<(), CompilerError> {
        let chunk = &mut function.chunk;
        match expr {
            Expression::Function(function_expression) => {
                let name = function_expression.name.to_string();
                let function_id = self.globals.insert(&name, None);
                dbg!(&self.globals);
                let mut child_function = Function::new(name.to_string());
                self.emit_function_expression(&mut child_function, &function_expression)?;
                let function_object = Object::new_with_name(name, Value::Function(child_function));
                self.globals.set(function_id, function_object);
                Ok(())
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
            Expression::Continue => self.emit_continue_statement(function),
            Expression::Break => self.emit_break_statement(function),
            Expression::Return(return_expression) => {
                self.emit_return_expression(function, &return_expression)
            }
            Expression::Assignment(assignment) => self.emit_assignment_op(function, &assignment),
            Expression::Unary(unary) => self.emit_unary_op(function, &unary),
            Expression::Binary(binary) => self.emit_binary_op(function, &binary),
            Expression::Variable(value) => self.emit_variable_op(function, &value),
            Expression::Literal(identifier) => self.emit_literal(chunk, &identifier),
            Expression::Empty => {
                chunk.emit(Bytecode::Nop);
                Ok(())
            }
        }
    }

    fn emit_function_expression(
        &mut self,
        function: &mut Function,
        function_expression: &FunctionExpression,
    ) -> Result<(), CompilerError> {
        function.arity = function_expression.args.len();
        function_expression.args.iter().for_each(|arg_name| {
            self.declare_local(arg_name);
        });
        self.emit_block_expression(function, &function_expression.body)
    }

    fn emit_call_expression(
        &mut self,
        function: &mut Function,
        call_expression: &CallExpression,
    ) -> Result<(), CompilerError> {
        // emit in reverse order to save processing in interpreting loop
        call_expression
            .args
            .iter()
            .rev()
            .try_for_each(|expr| self.emit_expression(function, expr.as_ref()))?;
        self.emit_expression(function, call_expression.callable.as_ref())?;
        function.chunk.emit(Bytecode::Call);
        function.chunk.emit_index(call_expression.args.len() as u64);
        Ok(())
    }

    fn emit_block_expression(
        &mut self,
        function: &mut Function,
        block_expr: &BlockExpression,
    ) -> Result<(), CompilerError> {
        self.begin_scope();
        block_expr
            .exprs
            .iter()
            .try_for_each(|expr| self.emit_expression(function, expr.as_ref()))?;
        self.end_scope(function);
        Ok(())
    }

    fn emit_if_expression(
        &mut self,
        function: &mut Function,
        if_expr: &IfExpression,
    ) -> Result<(), CompilerError> {
        // Emit If branch
        let mut exit_jump_addrs: Vec<u64> = Vec::new();

        let exit_jump_addr = self.emit_if_branch(
            function,
            if_expr.condition.as_ref(),
            if_expr.then_branch.as_ref(),
        )?;
        exit_jump_addrs.push(exit_jump_addr);

        // Emit Elif branches
        if_expr.elif_branches.iter().try_for_each(|elif_expr| {
            let exit_jump_addr = self.emit_if_branch(
                function,
                elif_expr.condition.as_ref(),
                elif_expr.then_branch.as_ref(),
            )?;
            exit_jump_addrs.push(exit_jump_addr);
            Ok(())
        })?;

        // Emit Else branch
        self.emit_expression(function, if_expr.else_branch.as_ref())?;

        // Patch exit addresses to prevent fallthrough
        let next_addr = function.chunk.size();
        exit_jump_addrs.iter().for_each(|addr| {
            function.chunk.patch_jump_addr(*addr, next_addr);
        });
        Ok(())
    }

    fn emit_if_branch(
        &mut self,
        function: &mut Function,
        condition: &Expression,
        then_branch: &Expression,
    ) -> Result<u64, CompilerError> {
        self.emit_expression(function, condition)?;
        function.chunk.emit(Bytecode::JumpIfFalse);
        let jump_offset_addr = function.chunk.emit_index(0);
        self.emit_expression(function, then_branch)?;
        function.chunk.emit(Bytecode::Jump);
        let exit_offset_addr = function.chunk.emit_index(0);
        function
            .chunk
            .patch_jump_addr(jump_offset_addr, function.chunk.size());
        Ok(exit_offset_addr)
    }

    fn emit_while_expression(
        &mut self,
        function: &mut Function,
        while_expr: &WhileExpression,
    ) -> Result<(), CompilerError> {
        // emit conditional
        let start_addr = function.chunk.size();

        self.continue_addr_stack.push(start_addr);
        let start_break_addr_stack_size = self.break_addr_stack.len();

        self.emit_expression(function, while_expr.condition.as_ref())?;
        function.chunk.emit(Bytecode::JumpIfFalse);
        let jump_offset_addr = function.chunk.emit_index(0);

        // emit body
        self.emit_expression(function, while_expr.body.as_ref())?;

        // next continue should not refer to this loop
        self.continue_addr_stack.pop();

        // loop to the beginning
        let chunk = &mut function.chunk;
        chunk.emit(Bytecode::Loop);
        chunk.emit_index(start_addr);

        // exit address
        let exit_addr = chunk.size();
        chunk.patch_jump_addr(jump_offset_addr, exit_addr);

        // patch break jumps
        while self.break_addr_stack.len() > start_break_addr_stack_size {
            let jump_offset_addr = self.break_addr_stack.pop().unwrap();
            chunk.patch_jump_addr(jump_offset_addr, exit_addr);
        }

        Ok(())
    }

    fn emit_continue_statement(&mut self, function: &mut Function) -> Result<(), CompilerError> {
        if self.continue_addr_stack.is_empty() {
            return Err(CompilerError::InvalidExpression(String::from(
                "continue without loop",
            )));
        }
        let loop_start_addr = self.continue_addr_stack.last().unwrap();
        function.chunk.emit(Bytecode::Loop);
        function.chunk.emit_index(*loop_start_addr);
        Ok(())
    }

    fn emit_break_statement(&mut self, function: &mut Function) -> Result<(), CompilerError> {
        function.chunk.emit(Bytecode::Jump);
        let break_offset_addr = function.chunk.emit_index(0);
        self.break_addr_stack.push(break_offset_addr);
        Ok(())
    }

    fn emit_return_expression(
        &mut self,
        function: &mut Function,
        return_expression: &ReturnExpression,
    ) -> Result<(), CompilerError> {
        match return_expression.expr.as_ref() {
            Expression::Empty => function.chunk.emit(Bytecode::None),
            _ => self.emit_expression(function, return_expression.expr.as_ref())?,
        };
        function.chunk.emit(Bytecode::Return);
        Ok(())
    }

    fn emit_assignment_op(
        &mut self,
        function: &mut Function,
        assignment_expr: &AssignmentExpression,
    ) -> Result<(), CompilerError> {
        self.emit_expression(function, assignment_expr.rhs.as_ref())?;
        match assignment_expr.lhs.as_ref() {
            Expression::Variable(variable_expr) => {
                if self.is_global_scope() {
                    let index = self.get_or_declare_global(variable_expr);
                    function.chunk.emit(Bytecode::SetGlobal);
                    function.chunk.emit_index(index);
                } else if function.is_global_scope() && self.globals.contains_name(variable_expr) {
                    let index = self.globals.get_index(variable_expr);
                    function.chunk.emit(Bytecode::SetGlobal);
                    function.chunk.emit_index(index);
                } else {
                    let index = self.get_or_declare_local(variable_expr);
                    function.chunk.emit(Bytecode::SetLocal);
                    function.chunk.emit_index(index);
                }
            }
            _ => {
                return Err(CompilerError::NameNotFound(String::from(
                    "Assignment must set a variable",
                )));
            }
        };
        Ok(())
    }

    fn emit_unary_op(
        &mut self,
        function: &mut Function,
        unary_expr: &UnaryExpression,
    ) -> Result<(), CompilerError> {
        self.emit_expression(function, unary_expr.expr.as_ref())?;
        self.emit_op(&mut function.chunk, &unary_expr.op)?;
        Ok(())
    }

    fn emit_binary_op(
        &mut self,
        function: &mut Function,
        binary_expr: &BinaryExpression,
    ) -> Result<(), CompilerError> {
        self.emit_expression(function, binary_expr.lhs.as_ref())?;
        self.emit_expression(function, binary_expr.rhs.as_ref())?;
        self.emit_op(&mut function.chunk, &binary_expr.op)?;
        Ok(())
    }

    fn emit_variable_op(
        &mut self,
        function: &mut Function,
        identifier: &String,
    ) -> Result<(), CompilerError> {
        if self.is_global_scope() {
            let index: u64;
            if self.globals.contains_name(identifier) {
                index = self.globals.get_index(identifier);
            } else {
                return Err(CompilerError::NameNotFound(format!(
                    "Name {} not found",
                    identifier
                )));
            }
            function.chunk.emit(Bytecode::GetGlobal);
            function.chunk.emit_index(index);
        } else {
            if function.is_global_scope() && self.globals.contains_name(identifier) {
                let index = self.globals.get_index(identifier);
                function.chunk.emit(Bytecode::GetGlobal);
                function.chunk.emit_index(index);
            } else if let Some(index) = self
                .locals
                .iter()
                .rposition(|local| &local.name == identifier)
            {
                function.chunk.emit(Bytecode::GetLocal);
                function.chunk.emit_index(index as u64);
            } else if &function.name == identifier {
                // recursive call
                let index = self.globals.get_index(identifier);
                function.chunk.emit(Bytecode::GetGlobal);
                function.chunk.emit_index(index);
            } else {
                return Err(CompilerError::NameNotFound(format!(
                    "Name {} not found",
                    identifier
                )));
            }
        }
        Ok(())
    }

    fn emit_op(&self, chunk: &mut Chunk, op: &Operator) -> Result<(), CompilerError> {
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
        Ok(())
    }

    fn emit_literal(&self, chunk: &mut Chunk, literal: &Literal) -> Result<(), CompilerError> {
        match literal {
            Literal::None => chunk.emit(Bytecode::None),
            Literal::True => chunk.emit(Bytecode::True),
            Literal::False => chunk.emit(Bytecode::False),
            _ => {
                let index = chunk.add_constant(literal);
                chunk.emit(Bytecode::Const);
                chunk.emit_index(index as u64);
            }
        }
        Ok(())
    }

    fn is_global_scope(&self) -> bool {
        self.scope_depth == 0
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn get_or_declare_global(&mut self, identifier: &String) -> u64 {
        if self.globals.contains_name(identifier) {
            self.globals.get_index(identifier)
        } else {
            self.globals.insert(identifier, None)
        }
    }

    fn get_or_declare_local(&mut self, identifier: &String) -> u64 {
        match self
            .locals
            .iter()
            .rposition(|local| &local.name == identifier)
        {
            Some(index) => index as u64,
            _ => self.declare_local(identifier),
        }
    }

    fn declare_local(&mut self, identifier: &String) -> u64 {
        self.locals.push(Local {
            name: identifier.to_string(),
            depth: self.scope_depth,
        });
        (self.locals.len() - 1) as u64
    }

    fn end_scope(&mut self, function: &mut Function) {
        self.scope_depth -= 1;

        while !self.locals.is_empty() && self.locals[self.locals.len() - 1].depth > self.scope_depth
        {
            function.chunk.emit(Bytecode::Pop);
            self.locals.pop();
        }
    }
}
