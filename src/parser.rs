use crate::ast::{
    AssignmentExpression, BinaryExpression, Expression, IfExpression, Literal, Operator, Program,
    UnaryExpression,
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
        if self.match_token(&Token::If) {
            self.parse_if_expression()
        } else {
            self.parse_assignment()
        }
    }

    fn parse_if_expression(&mut self) -> Result<Box<Expression>, ParserError> {
        let condition = self.parse_expression()?;
        let then_branch;
        if self.match_token(&Token::Indent) {
            then_branch = self.parse_expression()?;
        } else {
            return Err(ParserError::InvalidPrimary(format!(
                "If statement without body"
            )));
        }
        if !self.match_token(&Token::Colon) {
            return Err(ParserError::InvalidPrimary(String::from(
                "If statement missing colon ':'",
            )));
        }
        let else_branch = Box::new(Expression::Empty);
        Ok(Box::new(Expression::If(crate::ast::IfExpression {
            condition,
            then_branch,
            else_branch,
        })))
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
            let op = self.get_binary_operator(self.previous_token())?;
            let rhs = self.parse_equality()?;
            expr = Box::new(Expression::Binary(BinaryExpression { lhs: expr, op, rhs }));
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Box<Expression>, ParserError> {
        let mut expr = self.parse_comparison()?;

        while self.match_token(&Token::EqualEqual) || self.match_token(&Token::BangEqual) {
            let op = self.get_binary_operator(self.previous_token())?;
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
            let op = self.get_binary_operator(self.previous_token())?;
            let rhs = self.parse_term()?;
            expr = Box::new(Expression::Binary(BinaryExpression { lhs: expr, op, rhs }));
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Box<Expression>, ParserError> {
        let mut expr = self.parse_factor()?;

        while self.match_token(&Token::Plus) || self.match_token(&Token::Minus) {
            let op = self.get_binary_operator(self.previous_token())?;
            let rhs = self.parse_factor()?;
            expr = Box::new(Expression::Binary(BinaryExpression { lhs: expr, op, rhs }));
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Box<Expression>, ParserError> {
        let mut expr = self.parse_unary()?;

        while self.match_token(&Token::Star) || self.match_token(&Token::Slash) {
            let op = self.get_binary_operator(self.previous_token())?;
            let rhs = self.parse_unary()?;
            expr = Box::new(Expression::Binary(BinaryExpression { lhs: expr, op, rhs }));
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Box<Expression>, ParserError> {
        if self.match_token(&Token::Bang) || self.match_token(&Token::Minus) {
            let previous_token = self.previous_token();
            let op = match previous_token {
                Token::Bang => Operator::Not,
                Token::Minus => Operator::Neg,
                _ => {
                    return Err(ParserError::InvalidOperator(format!(
                        "Invalid unary operator: {:?}",
                        previous_token
                    )))
                }
            };
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
            Token::None => Ok(Box::new(Expression::Literal(Literal::None))),
            Token::True => Ok(Box::new(Expression::Literal(Literal::True))),
            Token::False => Ok(Box::new(Expression::Literal(Literal::False))),
            Token::Integer(value) => Ok(Box::new(Expression::Literal(Literal::Integer(*value)))),
            Token::Float(value) => Ok(Box::new(Expression::Literal(Literal::Float(*value)))),
            Token::String(value) => Ok(Box::new(Expression::Literal(Literal::String(
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

    fn get_binary_operator(&self, token: &Token) -> Result<Operator, ParserError> {
        match token {
            Token::And => Ok(Operator::And),
            Token::Or => Ok(Operator::Or),
            Token::EqualEqual => Ok(Operator::Equal),
            Token::BangEqual => Ok(Operator::NotEqual),
            Token::Bang => Ok(Operator::Not),
            Token::Less => Ok(Operator::Less),
            Token::LessEqual => Ok(Operator::LessEqual),
            Token::Greater => Ok(Operator::Greater),
            Token::GreaterEqual => Ok(Operator::GreaterEqual),
            Token::Plus => Ok(Operator::Add),
            Token::Minus => Ok(Operator::Sub),
            Token::Star => Ok(Operator::Mul),
            Token::Slash => Ok(Operator::Div),
            _ => Err(ParserError::InvalidOperator(format!(
                "Invalid binary operator: {:?}",
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
    fn test_primaries() {
        vec![
            (
                vec![Token::None, Token::Eof],
                vec![Box::new(Expression::Literal(Literal::None))],
            ),
            (
                vec![Token::True, Token::Eof],
                vec![Box::new(Expression::Literal(Literal::True))],
            ),
            (
                vec![Token::False, Token::Eof],
                vec![Box::new(Expression::Literal(Literal::False))],
            ),
            (
                vec![Token::Integer(1), Token::Eof],
                vec![Box::new(Expression::Literal(Literal::Integer(1)))],
            ),
            (
                vec![Token::Float(1.0), Token::Eof],
                vec![Box::new(Expression::Literal(Literal::Float(1.0)))],
            ),
            (
                vec![Token::String(String::from("test1")), Token::Eof],
                vec![Box::new(Expression::Literal(Literal::String(
                    String::from("test1"),
                )))],
            ),
            (
                vec![Token::Identifier(String::from("var1")), Token::Eof],
                vec![Box::new(Expression::Variable(String::from("var1")))],
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
    fn test_assignments() {
        vec![
            // Simple Assignment
            (
                vec![
                    Token::Identifier(String::from("var1")),
                    Token::Equal,
                    Token::True,
                    Token::Eof,
                ],
                vec![Box::new(Expression::Assignment(AssignmentExpression {
                    lhs: Box::new(Expression::Variable(String::from("var1"))),
                    rhs: Box::new(Expression::Literal(Literal::True)),
                }))],
            ),
            // Chained Assignment
            (
                vec![
                    Token::Identifier(String::from("var1")),
                    Token::Equal,
                    Token::Identifier(String::from("var2")),
                    Token::Equal,
                    Token::True,
                    Token::Eof,
                ],
                vec![Box::new(Expression::Assignment(AssignmentExpression {
                    lhs: Box::new(Expression::Variable(String::from("var1"))),
                    rhs: Box::new(Expression::Assignment(AssignmentExpression {
                        lhs: Box::new(Expression::Variable(String::from("var2"))),
                        rhs: Box::new(Expression::Literal(Literal::True)),
                    })),
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

    #[test]
    fn test_unary_expressions() {
        vec![
            (
                vec![Token::Bang, Token::True, Token::Eof],
                vec![Box::new(Expression::Unary(UnaryExpression {
                    op: Operator::Not,
                    expr: Box::new(Expression::Literal(Literal::True)),
                }))],
            ),
            (
                vec![Token::Minus, Token::Integer(1), Token::Eof],
                vec![Box::new(Expression::Unary(UnaryExpression {
                    op: Operator::Neg,
                    expr: Box::new(Expression::Literal(Literal::Integer(1))),
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

    #[test]
    fn test_binary_expressions() {
        vec![
            (
                vec![Token::True, Token::And, Token::False, Token::Eof],
                vec![Box::new(Expression::Binary(BinaryExpression {
                    lhs: Box::new(Expression::Literal(Literal::True)),
                    op: Operator::And,
                    rhs: Box::new(Expression::Literal(Literal::False)),
                }))],
            ),
            (
                vec![Token::True, Token::Or, Token::False, Token::Eof],
                vec![Box::new(Expression::Binary(BinaryExpression {
                    lhs: Box::new(Expression::Literal(Literal::True)),
                    op: Operator::Or,
                    rhs: Box::new(Expression::Literal(Literal::False)),
                }))],
            ),
            (
                vec![
                    Token::Integer(1),
                    Token::Less,
                    Token::Integer(2),
                    Token::Eof,
                ],
                vec![Box::new(Expression::Binary(BinaryExpression {
                    lhs: Box::new(Expression::Literal(Literal::Integer(1))),
                    op: Operator::Less,
                    rhs: Box::new(Expression::Literal(Literal::Integer(2))),
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
