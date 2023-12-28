use crate::ast::{
    AssignmentExpression, BinaryExpression, CallExpression, Expression, FunctionExpression,
    Operator, Program, UnaryExpression, Value,
};
use crate::token::Token;

#[derive(Clone, Debug)]
pub enum ParserError {
    InvalidOperator(String),
    InvalidPrimary(String),
}

pub struct Parser {
    tokens: Vec<Token>,
    index: usize,
    program: Program,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            index: 0,
            program: Program::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Program, ParserError> {
        self.index = 0;

        while self.index < self.tokens.len() && self.tokens[self.index] != Token::Eof {
            let expr = self.parse_expression()?;
            self.program.stmts.push(expr);
            self.index += 1;
        }

        Ok(self.program.clone())
    }

    fn parse_expression(&mut self) -> Result<Box<Expression>, ParserError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Box<Expression>, ParserError> {
        let expr = self.parse_logic_operator()?;

        if self.match_token(&Token::Equal) {
            let rhs = self.parse_assignment()?;
            return Ok(Box::new(Expression::Assignment(AssignmentExpression {
                lhs: expr,
                rhs,
            })));
        }

        Ok(expr)
    }

    fn parse_logic_operator(&mut self) -> Result<Box<Expression>, ParserError> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&Token::Or) || self.match_token(&Token::And) {
            let op = self.get_operator(self.previous_token())?;
            let rhs = self.parse_equality()?;
            expr = Box::new(Expression::Binary(BinaryExpression { lhs: expr, op, rhs }));
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Box<Expression>, ParserError> {
        let mut expr = self.parse_comparison()?;

        while self.match_token(&Token::EqualEqual) || self.match_token(&Token::BangEqual) {
            let op = self.get_operator(self.previous_token())?;
            let rhs = self.parse_comparison()?;
            expr = Box::new(Expression::Binary(BinaryExpression { lhs: expr, op, rhs }));
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Box<Expression>, ParserError> {
        let mut expr = self.parse_term()?;

        while self.match_token(&Token::Greater)
            || self.match_token(&Token::GreaterEqual)
            || self.match_token(&Token::Less)
            || self.match_token(&Token::LessEqual)
        {
            let op = self.get_operator(self.previous_token())?;
            let rhs = self.parse_term()?;
            expr = Box::new(Expression::Binary(BinaryExpression { lhs: expr, op, rhs }));
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Box<Expression>, ParserError> {
        let mut expr = self.parse_factor()?;

        while self.match_token(&Token::Plus) || self.match_token(&Token::Minus) {
            let op = self.get_operator(self.previous_token())?;
            let rhs = self.parse_factor()?;
            expr = Box::new(Expression::Binary(BinaryExpression { lhs: expr, op, rhs }));
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Box<Expression>, ParserError> {
        let mut expr = self.parse_unary()?;

        while self.match_token(&Token::Star) || self.match_token(&Token::Slash) {
            let op = self.get_operator(self.previous_token())?;
            let rhs = self.parse_unary()?;
            expr = Box::new(Expression::Binary(BinaryExpression { lhs: expr, op, rhs }));
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Box<Expression>, ParserError> {
        if self.match_token(&Token::Bang) || self.match_token(&Token::Minus) {
            let op = self.get_operator(self.previous_token())?;
            let rhs = self.parse_unary()?;
            return Ok(Box::new(Expression::Unary(UnaryExpression {
                op,
                expr: rhs,
            })));
        }
        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Box<Expression>, ParserError> {
        let expr = self.parse_primary()?;
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Box<Expression>, ParserError> {
        let expr = match self.current_token() {
            Token::None => Ok(Box::new(Expression::Literal(Value::None))),
            Token::True => Ok(Box::new(Expression::Literal(Value::Bool(true)))),
            Token::False => Ok(Box::new(Expression::Literal(Value::Bool(false)))),
            Token::Numeric(value) => Ok(Box::new(Expression::Literal(Value::Numeric(*value)))),
            Token::String(value) => Ok(Box::new(Expression::Literal(Value::String(
                value.to_string(),
            )))),
            Token::Identifier(value) => Ok(Box::new(Expression::Variable(value.to_string()))),
            other => Err(ParserError::InvalidPrimary(format!(
                "Invalid primary: {:?}",
                other
            ))),
        };
        self.advance_token();
        expr
    }

    fn get_operator(&self, token: &Token) -> Result<Operator, ParserError> {
        match token {
            Token::And => Ok(Operator::And),
            Token::Or => Ok(Operator::Or),
            Token::EqualEqual => Ok(Operator::Equal),
            Token::BangEqual => Ok(Operator::NotEqual),
            Token::Bang => Ok(Operator::Neg),
            Token::Less => Ok(Operator::Less),
            Token::LessEqual => Ok(Operator::LessEqual),
            Token::Greater => Ok(Operator::Greater),
            Token::GreaterEqual => Ok(Operator::GreaterEqual),
            Token::Plus => Ok(Operator::Add),
            Token::Minus => Ok(Operator::Sub),
            Token::Star => Ok(Operator::Mul),
            Token::Slash => Ok(Operator::Div),
            _ => Err(ParserError::InvalidOperator(format!(
                "Invalid Operator: {:?}",
                token
            ))),
        }
    }

    fn previous_token(&self) -> &Token {
        if let Some(tok) = self.tokens.get(self.index - 1) {
            tok
        } else {
            &Token::Empty
        }
    }

    fn current_token(&self) -> &Token {
        if let Some(tok) = self.tokens.get(self.index) {
            tok
        } else {
            &Token::Eof
        }
    }

    fn match_token(&mut self, token: &Token) -> bool {
        if let Some(tok) = self.tokens.get(self.index) {
            if tok == token {
                self.advance_token();
                return true;
            }
        }
        false
    }

    fn advance_token(&mut self) {
        if self.index + 1 < self.tokens.len() {
            self.index += 1;
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_literals() {
        vec![
            (
                vec![Token::None, Token::Eof],
                vec![Box::new(Expression::Literal(Value::None))],
            ),
            (
                vec![Token::True, Token::Eof],
                vec![Box::new(Expression::Literal(Value::Bool(true)))],
            ),
            (
                vec![Token::False, Token::Eof],
                vec![Box::new(Expression::Literal(Value::Bool(false)))],
            ),
            (
                vec![Token::Numeric(1.23), Token::Eof],
                vec![Box::new(Expression::Literal(Value::Numeric(1.23)))],
            ),
            (
                vec![Token::String(String::from("test1")), Token::Eof],
                vec![Box::new(Expression::Literal(Value::String(String::from(
                    "test1",
                ))))],
            ),
        ]
        .into_iter()
        .for_each(|(tokens, exprs)| {
            let mut parser = Parser::new(tokens);
            let program = match parser.parse() {
                Ok(program) => program,
                Err(err) => panic!("ParseError: {:?}", err),
            };
            assert_eq!(program.stmts, exprs);
        });
    }

    #[test]
    fn test_binary_expressions() {
        vec![
            (
                vec![Token::True, Token::And, Token::False, Token::Eof],
                vec![Box::new(Expression::Binary(BinaryExpression {
                    lhs: Box::new(Expression::Literal(Value::Bool(true))),
                    op: Operator::And,
                    rhs: Box::new(Expression::Literal(Value::Bool(false))),
                }))],
            ),
            (
                vec![Token::True, Token::Or, Token::False, Token::Eof],
                vec![Box::new(Expression::Binary(BinaryExpression {
                    lhs: Box::new(Expression::Literal(Value::Bool(true))),
                    op: Operator::Or,
                    rhs: Box::new(Expression::Literal(Value::Bool(false))),
                }))],
            ),
            (
                vec![
                    Token::Numeric(1.0),
                    Token::Less,
                    Token::Numeric(2.0),
                    Token::Eof,
                ],
                vec![Box::new(Expression::Binary(BinaryExpression {
                    lhs: Box::new(Expression::Literal(Value::Numeric(1.0))),
                    op: Operator::Less,
                    rhs: Box::new(Expression::Literal(Value::Numeric(2.0))),
                }))],
            ),
        ]
        .into_iter()
        .for_each(|(tokens, exprs)| {
            let mut parser = Parser::new(tokens);
            let program = match parser.parse() {
                Ok(program) => program,
                Err(err) => panic!("ParseError: {:?}", err),
            };
            assert_eq!(program.stmts, exprs);
        });
    }
}
