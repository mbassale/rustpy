use crate::ast::{
    AssignmentExpression, BinaryExpression, BlockExpression, CallExpression, ElifExpression,
    Expression, FunctionExpression, IfExpression, Literal, Operator, PrintExpression, Program,
    ReturnExpression, UnaryExpression, WhileExpression,
};
use crate::token::Token;

#[derive(Clone, Debug)]
pub enum ParserError {
    InvalidOperator(String),
    InvalidPrimary(String),
    InvalidExpression(String),
}

pub struct Parser {
    tokens: Vec<Token>,
    index: usize,
    program: Program,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            // filter Token::NewLine
            tokens: tokens
                .into_iter()
                .filter(|tok| tok != &Token::NewLine)
                .collect(),
            index: 0,
            program: Program::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Program, ParserError> {
        self.index = 0;

        while self.index < self.tokens.len() && self.tokens[self.index] != Token::Eof {
            let expr = self.parse_expression()?;
            self.program.stmts.push(expr);
        }

        Ok(self.program.clone())
    }

    fn parse_expression(&mut self) -> Result<Box<Expression>, ParserError> {
        if self.match_token(&Token::Indent) {
            self.parse_block_expression()
        } else if self.match_token(&Token::Def) {
            self.parse_function_expression()
        } else if self.match_token(&Token::If) {
            self.parse_if_expression()
        } else if self.match_token(&Token::While) {
            self.parse_while_expression()
        } else if self.match_token(&Token::Return) {
            self.parse_return_expression()
        } else if self.match_token(&Token::Print) {
            self.parse_print_expression()
        } else {
            self.parse_assignment()
        }
    }

    fn parse_block_expression(&mut self) -> Result<Box<Expression>, ParserError> {
        let mut exprs: Vec<Box<Expression>> = Vec::new();
        while self.index < self.tokens.len()
            && self.tokens[self.index] != Token::Dedent
            && self.tokens[self.index] != Token::Eof
        {
            let expr = self.parse_expression()?;
            exprs.push(expr);
        }
        self.match_token(&Token::Dedent);
        Ok(Box::new(Expression::Block(BlockExpression { exprs })))
    }

    fn parse_function_expression(&mut self) -> Result<Box<Expression>, ParserError> {
        let function_name = match self.current_token() {
            Token::Identifier(function_name) => function_name.to_string(),
            _ => {
                return Err(ParserError::InvalidExpression(String::from(
                    "Missing function name",
                )))
            }
        };
        self.advance_token();

        if !self.match_token(&Token::LeftParen) {
            return Err(ParserError::InvalidExpression(String::from("Missing '('")));
        }

        let mut args: Vec<String> = Vec::new();
        loop {
            let arg_name = match self.current_token() {
                Token::Identifier(arg_name) => arg_name.to_string(),
                Token::RightParen => {
                    self.advance_token();
                    break;
                }
                _ => break,
            };
            args.push(arg_name);
            self.advance_token();

            if self.match_token(&Token::Comma) || self.match_token(&Token::RightParen) {
                if self.previous_token() == &Token::RightParen {
                    break;
                }
            } else {
                return Err(ParserError::InvalidExpression(String::from(
                    "Expected argument or ')'",
                )));
            }
        }

        if !self.match_token(&Token::Colon) {
            return Err(ParserError::InvalidExpression(String::from("Missing ':'")));
        }

        let body_expr = self.parse_expression()?;
        let block_expression = match *body_expr {
            Expression::Block(block_expression) => block_expression,
            _ => {
                return Err(ParserError::InvalidExpression(String::from(
                    "Bad function definition, expected block",
                )))
            }
        };

        Ok(Box::new(Expression::Function(FunctionExpression {
            name: function_name,
            args,
            body: block_expression,
        })))
    }

    fn parse_if_expression(&mut self) -> Result<Box<Expression>, ParserError> {
        let mut if_expression = IfExpression {
            condition: Box::new(Expression::Empty),
            then_branch: Box::new(Expression::Empty),
            elif_branches: Vec::new(),
            else_branch: Box::new(Expression::Empty),
        };
        // Parse conditional
        if_expression.condition = self.parse_expression()?;

        // Check colon ':'
        if !self.match_token(&Token::Colon) {
            return Err(ParserError::InvalidExpression(String::from(
                "If expression missing colon ':'",
            )));
        }

        // Required a then branch
        if self.match_token(&Token::Indent) {
            if_expression.then_branch = self.parse_block_expression()?;
        } else {
            return Err(ParserError::InvalidExpression(format!(
                "If expression without body"
            )));
        }

        // Optional Elif branches
        while self.match_token(&Token::Elif) {
            let condition = self.parse_expression()?;
            if self.match_token(&Token::Colon) {
                if self.match_token(&Token::Indent) {
                    let then_branch = self.parse_block_expression()?;
                    if_expression.elif_branches.push(ElifExpression {
                        condition,
                        then_branch,
                    });
                } else {
                    return Err(ParserError::InvalidExpression(String::from(
                        "Elif expression without body",
                    )));
                }
            } else {
                return Err(ParserError::InvalidExpression(String::from(
                    "Elif expression missing colon ':'",
                )));
            }
        }

        // Optional Else branch
        if self.match_token(&Token::Else) {
            if self.match_token(&Token::Colon) {
                if self.match_token(&Token::Indent) {
                    if_expression.else_branch = self.parse_block_expression()?;
                } else {
                    return Err(ParserError::InvalidExpression(String::from(
                        "Else expression without body",
                    )));
                }
            } else {
                return Err(ParserError::InvalidExpression(String::from(
                    "Else expression missing colon ':'",
                )));
            }
        }

        Ok(Box::new(Expression::If(if_expression)))
    }

    fn parse_while_expression(&mut self) -> Result<Box<Expression>, ParserError> {
        let condition = self.parse_expression()?;

        let body;
        if self.match_token(&Token::Colon) {
            body = self.parse_expression()?;
        } else {
            return Err(ParserError::InvalidExpression(String::from(
                "While expression missing colon ':'",
            )));
        }

        Ok(Box::new(Expression::While(WhileExpression {
            condition,
            body,
        })))
    }

    fn parse_return_expression(&mut self) -> Result<Box<Expression>, ParserError> {
        let expr = match self.current_token() {
            &Token::Dedent | &Token::Eof => Box::new(Expression::Empty),
            _ => self.parse_assignment()?,
        };
        Ok(Box::new(Expression::Return(ReturnExpression { expr })))
    }

    fn parse_print_expression(&mut self) -> Result<Box<Expression>, ParserError> {
        let expr = self.parse_expression()?;
        Ok(Box::new(Expression::Print(PrintExpression { expr })))
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

        let mut args: Vec<Box<Expression>> = Vec::new();
        if self.match_token(&Token::LeftParen) {
            while !self.match_token(&Token::RightParen) {
                let argument = self.parse_expression()?;
                args.push(argument);
                self.match_token(&Token::Comma);
            }
            return Ok(Box::new(Expression::Call(CallExpression {
                callable: expr,
                args,
            })));
        }

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

    #[test]
    fn test_if_statement() {
        vec![
            // If
            (
                vec![
                    Token::If,
                    Token::True,
                    Token::Colon,
                    Token::Indent,
                    Token::True,
                    Token::Eof,
                ],
                vec![Box::new(Expression::If(IfExpression {
                    condition: Box::new(Expression::Literal(Literal::True)),
                    then_branch: Box::new(Expression::Block(BlockExpression {
                        exprs: vec![Box::new(Expression::Literal(Literal::True))],
                    })),
                    elif_branches: Vec::new(),
                    else_branch: Box::new(Expression::Empty),
                }))],
            ),
            // If-Else
            (
                vec![
                    Token::If,
                    Token::True,
                    Token::Colon,
                    Token::NewLine,
                    Token::Indent,
                    Token::True,
                    Token::Dedent,
                    Token::Else,
                    Token::Colon,
                    Token::Indent,
                    Token::False,
                    Token::Dedent,
                    Token::Eof,
                ],
                vec![Box::new(Expression::If(IfExpression {
                    condition: Box::new(Expression::Literal(Literal::True)),
                    then_branch: Box::new(Expression::Block(BlockExpression {
                        exprs: vec![Box::new(Expression::Literal(Literal::True))],
                    })),
                    elif_branches: Vec::new(),
                    else_branch: Box::new(Expression::Block(BlockExpression {
                        exprs: vec![Box::new(Expression::Literal(Literal::False))],
                    })),
                }))],
            ),
            // If-Elif-Elif-Else
            (
                vec![
                    // if True:
                    Token::If,
                    Token::True,
                    Token::Colon,
                    Token::Indent,
                    Token::True,
                    Token::Dedent,
                    // elif False:
                    Token::Elif,
                    Token::False,
                    Token::Colon,
                    Token::Indent,
                    Token::False,
                    Token::Dedent,
                    // elif True:
                    Token::Elif,
                    Token::True,
                    Token::Colon,
                    Token::Indent,
                    Token::True,
                    Token::Dedent,
                    // else:
                    Token::Else,
                    Token::Colon,
                    Token::Indent,
                    Token::False,
                    Token::Dedent,
                    Token::Eof,
                ],
                vec![Box::new(Expression::If(IfExpression {
                    condition: Box::new(Expression::Literal(Literal::True)),
                    then_branch: Box::new(Expression::Block(BlockExpression {
                        exprs: vec![Box::new(Expression::Literal(Literal::True))],
                    })),
                    elif_branches: vec![
                        ElifExpression {
                            condition: Box::new(Expression::Literal(Literal::False)),
                            then_branch: Box::new(Expression::Block(BlockExpression {
                                exprs: vec![Box::new(Expression::Literal(Literal::False))],
                            })),
                        },
                        ElifExpression {
                            condition: Box::new(Expression::Literal(Literal::True)),
                            then_branch: Box::new(Expression::Block(BlockExpression {
                                exprs: vec![Box::new(Expression::Literal(Literal::True))],
                            })),
                        },
                    ],
                    else_branch: Box::new(Expression::Block(BlockExpression {
                        exprs: vec![Box::new(Expression::Literal(Literal::False))],
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
    fn test_while_expression() {
        vec![(
            vec![
                Token::While,
                Token::True,
                Token::Colon,
                Token::NewLine,
                Token::Indent,
                Token::True,
                Token::True,
                Token::Dedent,
                Token::Eof,
            ],
            vec![Box::new(Expression::While(WhileExpression {
                condition: Box::new(Expression::Literal(Literal::True)),
                body: Box::new(Expression::Block(BlockExpression {
                    exprs: vec![
                        Box::new(Expression::Literal(Literal::True)),
                        Box::new(Expression::Literal(Literal::True)),
                    ],
                })),
            }))],
        )]
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
    fn test_function_and_call_expressions() {
        vec![
            (
                vec![
                    Token::Def,
                    Token::Identifier(String::from("test")),
                    Token::LeftParen,
                    Token::RightParen,
                    Token::Colon,
                    Token::NewLine,
                    Token::Indent,
                    Token::Return,
                    Token::True,
                    Token::Dedent,
                    Token::Eof,
                ],
                vec![Box::new(Expression::Function(FunctionExpression {
                    name: String::from("test"),
                    args: vec![],
                    body: BlockExpression {
                        exprs: vec![Box::new(Expression::Return(ReturnExpression {
                            expr: Box::new(Expression::Literal(Literal::True)),
                        }))],
                    },
                }))],
            ),
            (
                vec![
                    Token::Def,
                    Token::Identifier(String::from("test")),
                    Token::LeftParen,
                    Token::Identifier(String::from("arg1")),
                    Token::Comma,
                    Token::Identifier(String::from("arg2")),
                    Token::RightParen,
                    Token::Colon,
                    Token::NewLine,
                    Token::Indent,
                    Token::Return,
                    Token::True,
                    Token::Dedent,
                    Token::Eof,
                ],
                vec![Box::new(Expression::Function(FunctionExpression {
                    name: String::from("test"),
                    args: vec![String::from("arg1"), String::from("arg2")],
                    body: BlockExpression {
                        exprs: vec![Box::new(Expression::Return(ReturnExpression {
                            expr: Box::new(Expression::Literal(Literal::True)),
                        }))],
                    },
                }))],
            ),
            (
                vec![
                    Token::Def,
                    Token::Identifier(String::from("test")),
                    Token::LeftParen,
                    Token::Identifier(String::from("arg1")),
                    Token::Comma,
                    Token::Identifier(String::from("arg2")),
                    Token::RightParen,
                    Token::Colon,
                    Token::NewLine,
                    Token::Indent,
                    Token::Return,
                    Token::Dedent,
                    Token::Eof,
                ],
                vec![Box::new(Expression::Function(FunctionExpression {
                    name: String::from("test"),
                    args: vec![String::from("arg1"), String::from("arg2")],
                    body: BlockExpression {
                        exprs: vec![Box::new(Expression::Return(ReturnExpression {
                            expr: Box::new(Expression::Empty),
                        }))],
                    },
                }))],
            ),
            (
                vec![
                    Token::Def,
                    Token::Identifier(String::from("test")),
                    Token::LeftParen,
                    Token::Identifier(String::from("arg1")),
                    Token::Comma,
                    Token::Identifier(String::from("arg2")),
                    Token::RightParen,
                    Token::Colon,
                    Token::NewLine,
                    Token::Indent,
                    Token::Return,
                    Token::Identifier(String::from("arg1")),
                    Token::Plus,
                    Token::Identifier(String::from("arg2")),
                    Token::Dedent,
                    Token::NewLine,
                    Token::Identifier(String::from("test")),
                    Token::LeftParen,
                    Token::Integer(1),
                    Token::Comma,
                    Token::Integer(2),
                    Token::RightParen,
                    Token::Eof,
                ],
                vec![
                    Box::new(Expression::Function(FunctionExpression {
                        name: String::from("test"),
                        args: vec![String::from("arg1"), String::from("arg2")],
                        body: BlockExpression {
                            exprs: vec![Box::new(Expression::Return(ReturnExpression {
                                expr: Box::new(Expression::Binary(BinaryExpression {
                                    lhs: Box::new(Expression::Variable(String::from("arg1"))),
                                    op: Operator::Add,
                                    rhs: Box::new(Expression::Variable(String::from("arg2"))),
                                })),
                            }))],
                        },
                    })),
                    Box::new(Expression::Call(CallExpression {
                        callable: Box::new(Expression::Variable(String::from("test"))),
                        args: vec![
                            Box::new(Expression::Literal(Literal::Integer(1))),
                            Box::new(Expression::Literal(Literal::Integer(2))),
                        ],
                    })),
                ],
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
    fn test_print_expression() {
        vec![
            (
                vec![Token::Print, Token::True, Token::Eof],
                vec![Box::new(Expression::Print(PrintExpression {
                    expr: Box::new(Expression::Literal(Literal::True)),
                }))],
            ),
            (
                vec![
                    Token::Print,
                    Token::Identifier(String::from("test1")),
                    Token::Plus,
                    Token::Identifier(String::from("test2")),
                    Token::Eof,
                ],
                vec![Box::new(Expression::Print(PrintExpression {
                    expr: Box::new(Expression::Binary(BinaryExpression {
                        lhs: Box::new(Expression::Variable(String::from("test1"))),
                        op: Operator::Add,
                        rhs: Box::new(Expression::Variable(String::from("test2"))),
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
}
