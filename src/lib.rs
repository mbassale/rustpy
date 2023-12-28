mod lexer;
mod token;

use crate::lexer::Lexer;
use crate::token::Token;

pub struct Interpreter {
    source: String,
    errors: Vec<String>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            source: String::new(),
            errors: vec![],
        }
    }

    pub fn run(&mut self, source: &str) {
        self.source = String::from(source);
        self.errors.clear();

        let tokens: Vec<Token> = Lexer::new(&self.source).into_iter().collect();
        dbg!(tokens);
    }
}
