mod ast;
mod bytecode;
mod chunk;
mod compiler;
mod disassembler;
mod function;
mod lexer;
pub mod object;
mod parser;
mod symbol_table;
mod token;
mod vm;

use symbol_table::SymbolTable;

use crate::compiler::{Compiler, CompilerError};
use crate::disassembler::Disassembler;
use crate::lexer::Lexer;
use crate::object::Value;
use crate::parser::{Parser, ParserError};
use crate::token::Token;
use crate::vm::{Vm, VmError};

#[derive(Clone, Debug)]
pub enum InterpreterError {
    LexerError(String),
    ParserError(ParserError),
    CompilerError(CompilerError),
    VmError(VmError),
}

pub struct Interpreter {
    globals: SymbolTable,
    source: String,
    vm: Vm,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            globals: SymbolTable::new(),
            source: String::new(),
            vm: Vm::new(),
        }
    }

    pub fn run(&mut self, source: &str) -> Result<Value, InterpreterError> {
        self.source = String::from(source);

        let tokens: Vec<Token> = Lexer::new(&self.source).into_iter().collect();
        let tokens = dbg!(tokens);
        self.check_lexer_errors(&tokens)?;

        let mut parser = Parser::new(tokens);
        let program = match parser.parse() {
            Ok(program) => program,
            Err(parser_error) => return Err(InterpreterError::ParserError(parser_error)),
        };
        let program = dbg!(program);

        let mut compiler = Compiler::new(program, &mut self.globals);
        let function = match compiler.compile() {
            Ok(function) => function,
            Err(compiler_error) => return Err(InterpreterError::CompilerError(compiler_error)),
        };
        let function = dbg!(function);

        let disassembler = Disassembler::new(function.chunk.clone());
        let instructions = disassembler.disassemble();
        dbg!(instructions);

        let result = match self.vm.interpret(&mut self.globals, function) {
            Ok(result) => result,
            Err(vm_error) => return Err(InterpreterError::VmError(vm_error)),
        };

        Ok(result.value)
    }

    fn check_lexer_errors(&self, tokens: &Vec<Token>) -> Result<(), InterpreterError> {
        if let Some(token_error) = tokens.iter().find(|&token| match token {
            Token::Error(_) => true,
            _ => false,
        }) {
            let error_message: String = match token_error {
                Token::Error(error_message) => error_message.to_string(),
                _ => String::from("Unknown lexer error"),
            };
            return Err(InterpreterError::LexerError(error_message));
        }
        Ok(())
    }
}
