mod ast;
mod bytecode;
mod compiler;
mod lexer;
mod parser;
mod token;
mod vm;

use crate::compiler::{Compiler, CompilerError};
use crate::lexer::Lexer;
use crate::parser::{Parser, ParserError};
use crate::token::Token;
use crate::vm::{Vm, VmError};

#[derive(Clone, Debug)]
pub enum InterpreterError {
    LexerError(String),
    ParserError(ParserError),
    CompilerError(CompilerError),
}

pub struct Interpreter {
    source: String,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            source: String::new(),
        }
    }

    pub fn run(&mut self, source: &str) -> Result<(), InterpreterError> {
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

        let mut compiler = Compiler::new(program);
        let chunk = match compiler.compile() {
            Ok(chunk) => chunk,
            Err(compiler_error) => return Err(InterpreterError::CompilerError(compiler_error)),
        };
        let chunk = dbg!(chunk);

        let mut vm = Vm::new();
        let _ = vm.interpret(&chunk);

        Ok(())
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
