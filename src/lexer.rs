use crate::token::Token;

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.chars.len() {
            match self.last_token {
                Token::Eof => return None,
                _ => {
                    self.last_token = Token::Eof;
                    return Some(Token::Eof);
                }
            }
        }

        self.last_token = self.parse_token();
        Some(self.last_token.clone())
    }
}

pub struct Lexer {
    index: usize,
    chars: Vec<char>,
    indentation_level: Token,
    last_token: Token,
}

impl Lexer {
    pub fn new(source: &str) -> Lexer {
        Lexer {
            index: 0,
            chars: source.chars().collect(),
            indentation_level: Token::Indent(0),
            last_token: Token::Empty,
        }
    }

    fn parse_token(&mut self) -> Token {
        let mut chr: char = match self.chars.get(self.index) {
            Some(chr) => *chr,
            None => return Token::Eof,
        };

        if chr.is_whitespace() {
            let start: usize = self.index;
            while chr.is_whitespace() {
                self.index += 1;
                if chr == '\n' {
                    return Token::NewLine;
                }
                chr = match self.chars.get(self.index) {
                    Some(chr) => *chr,
                    None => return Token::Eof,
                };
            }
            if self.last_token == Token::NewLine {
                let new_indentation_level = self.index - start;
                let current_indentation_level = match self.indentation_level {
                    Token::Indent(level) => level,
                    Token::Dedent(level) => level,
                    _ => unreachable!("indentation_level has an incorrect token"),
                };
                if new_indentation_level > current_indentation_level {
                    self.indentation_level = Token::Indent(new_indentation_level);
                    return self.indentation_level.clone();
                } else if new_indentation_level < current_indentation_level {
                    self.indentation_level = Token::Dedent(new_indentation_level);
                    return self.indentation_level.clone();
                }
            }
        }

        if chr == '"' {
            return self.parse_string(chr);
        }

        if chr.is_ascii_punctuation() {
            match self.parse_operator(chr) {
                Some(token) => return token,
                None => (),
            };
        }

        if chr.is_digit(10) {
            return self.parse_numeric(chr);
        }

        if chr == '_' || chr.is_alphabetic() {
            match self.parse_keyword(chr) {
                Some(token) => return token,
                None => (),
            }

            return self.parse_identifier(chr);
        }

        return Token::Error(format!("Error: invalid character: {}", chr));
    }

    fn parse_string(&mut self, chr: char) -> Token {
        let mut buffer = String::new();
        self.index += 1;
        loop {
            let c: char = match self.chars.get(self.index) {
                Some(c) => *c,
                None => '"',
            };
            if c == '"' {
                self.index += 1;
                break;
            }
            buffer.push(c);
            self.index += 1;
        }
        Token::String(buffer)
    }

    fn parse_operator(&mut self, chr: char) -> Option<Token> {
        let op = match chr {
            '+' => Some(Token::Plus),
            '-' => Some(Token::Minus),
            '*' => Some(Token::Star),
            '/' => Some(Token::Slash),
            ':' => Some(Token::Colon),
            ',' => Some(Token::Comma),
            '.' => Some(Token::Dot),
            '(' => Some(Token::LeftParen),
            ')' => Some(Token::RightParen),
            '[' => Some(Token::LeftBracket),
            ']' => Some(Token::RightBracket),
            '{' => Some(Token::LeftBrace),
            '}' => Some(Token::RightBrace),
            '!' => match self.chars.get(self.index + 1) {
                Some(&'=') => {
                    self.index += 1;
                    Some(Token::BangEqual)
                }
                _ => Some(Token::Bang),
            },
            '<' => match self.chars.get(self.index + 1) {
                Some(&'=') => {
                    self.index += 1;
                    Some(Token::LessEqual)
                }
                _ => Some(Token::Less),
            },
            '>' => match self.chars.get(self.index + 1) {
                Some(&'=') => {
                    self.index += 1;
                    Some(Token::GreaterEqual)
                }
                _ => Some(Token::Greater),
            },
            '=' => match self.chars.get(self.index + 1) {
                Some(&'=') => {
                    self.index += 1;
                    Some(Token::EqualEqual)
                }
                _ => Some(Token::Equal),
            },
            _ => None,
        };
        match op {
            Some(op) => {
                self.index += 1;
                Some(op)
            }
            None => None,
        }
    }

    fn parse_keyword(&mut self, chr: char) -> Option<Token> {
        match chr {
            'a' => self.consume(Token::And, "and"),
            'o' => self.consume(Token::Or, "or"),
            'i' => match self.chars.get(self.index + 1) {
                Some('f') => self.consume(Token::If, "if"),
                Some('n') => self.consume(Token::In, "in"),
                _ => None,
            },
            'e' => match self.chars.get(self.index + 1) {
                Some('l') => match self.chars.get(self.index + 2) {
                    Some('i') => self.consume(Token::Elif, "elif"),
                    Some('s') => self.consume(Token::Else, "else"),
                    _ => None,
                },
                _ => None,
            },
            'd' => self.consume(Token::Def, "def"),
            'f' => self.consume(Token::For, "for"),
            'F' => self.consume(Token::False, "False"),
            'N' => self.consume(Token::None, "None"),
            'w' => self.consume(Token::While, "while"),
            'c' => self.consume(Token::Continue, "continue"),
            'b' => self.consume(Token::Break, "break"),
            'r' => self.consume(Token::Return, "return"),
            'T' => self.consume(Token::True, "True"),
            _ => None,
        }
    }

    fn parse_identifier(&mut self, chr: char) -> Token {
        let mut buffer = String::new();
        let mut c = chr;
        let mut idx = self.index;
        while c.is_alphanumeric() || c == '_' {
            buffer.push(c);
            idx += 1;
            c = match self.chars.get(idx) {
                Some(c) => *c,
                None => break,
            };
        }
        self.index += buffer.len();
        Token::Identifier(buffer)
    }

    fn parse_numeric(&mut self, chr: char) -> Token {
        let mut buffer = String::new();
        let mut c = chr;
        let mut idx = self.index;
        while c.is_numeric() || c == '.' {
            buffer.push(c);
            idx += 1;
            c = match self.chars.get(idx) {
                Some(c) => *c,
                None => break,
            };
        }
        self.index += buffer.len();
        match buffer.find('.') {
            Some(_) => {
                let number: f64 = match buffer.parse() {
                    Ok(number) => number,
                    Err(_) => {
                        return Token::Error(format!("Invalid float: {}", buffer));
                    }
                };
                Token::Float(number)
            }
            None => {
                let number: i64 = match buffer.parse() {
                    Ok(number) => number,
                    Err(_) => {
                        return Token::Error(format!("Invalid integer: {}", buffer));
                    }
                };
                Token::Integer(number)
            }
        }
    }

    fn consume(&mut self, token: Token, keyword: &str) -> Option<Token> {
        if self.index + keyword.len() > self.chars.len() {
            return None;
        }
        let sub_str: String = self.chars[self.index..(self.index + keyword.len())]
            .into_iter()
            .collect();
        if sub_str == keyword.to_string() {
            self.index += keyword.len();
            Some(token)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operators() {
        let test_cases = vec![(
            "+-*/:,.()[]{}===<=<>=>!=!",
            vec![
                Token::Plus,
                Token::Minus,
                Token::Star,
                Token::Slash,
                Token::Colon,
                Token::Comma,
                Token::Dot,
                Token::LeftParen,
                Token::RightParen,
                Token::LeftBracket,
                Token::RightBracket,
                Token::LeftBrace,
                Token::RightBrace,
                Token::EqualEqual,
                Token::Equal,
                Token::LessEqual,
                Token::Less,
                Token::GreaterEqual,
                Token::Greater,
                Token::BangEqual,
                Token::Bang,
                Token::Eof,
            ],
        )];
        for (source, expected) in test_cases {
            let actual: Vec<Token> = Lexer::new(source).into_iter().collect();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_keywords() {
        vec![
            ("and", vec![Token::And, Token::Eof]),
            ("or", vec![Token::Or, Token::Eof]),
            ("if", vec![Token::If, Token::Eof]),
            ("in", vec![Token::In, Token::Eof]),
            ("def", vec![Token::Def, Token::Eof]),
            ("elif", vec![Token::Elif, Token::Eof]),
            ("else", vec![Token::Else, Token::Eof]),
            ("continue", vec![Token::Continue, Token::Eof]),
            ("break", vec![Token::Break, Token::Eof]),
            ("return", vec![Token::Return, Token::Eof]),
            ("None", vec![Token::None, Token::Eof]),
            ("True", vec![Token::True, Token::Eof]),
            ("False", vec![Token::False, Token::Eof]),
            (
                "and or if in def elif else continue break return None True False",
                vec![
                    Token::And,
                    Token::Or,
                    Token::If,
                    Token::In,
                    Token::Def,
                    Token::Elif,
                    Token::Else,
                    Token::Continue,
                    Token::Break,
                    Token::Return,
                    Token::None,
                    Token::True,
                    Token::False,
                    Token::Eof,
                ],
            ),
        ]
        .into_iter()
        .for_each(|(source, expected)| {
            let actual: Vec<Token> = Lexer::new(source).into_iter().collect();
            assert_eq!(actual, expected);
        });
    }

    #[test]
    fn test_identifiers() {
        vec![(
            "if test s98_foo_bar elif s_100 else _",
            vec![
                Token::If,
                Token::Identifier(String::from("test")),
                Token::Identifier(String::from("s98_foo_bar")),
                Token::Elif,
                Token::Identifier(String::from("s_100")),
                Token::Else,
                Token::Identifier(String::from("_")),
                Token::Eof,
            ],
        )]
        .into_iter()
        .for_each(|(source, expected)| {
            let actual: Vec<Token> = Lexer::new(source).into_iter().collect();
            assert_eq!(actual, expected);
        });
    }

    #[test]
    fn test_numbers() {
        vec![
            ("0", vec![Token::Integer(0), Token::Eof]),
            ("1", vec![Token::Integer(1), Token::Eof]),
            ("1234567890", vec![Token::Integer(1234567890), Token::Eof]),
            ("1.1", vec![Token::Float(1.1), Token::Eof]),
            ("1.23456789", vec![Token::Float(1.23456789), Token::Eof]),
            ("12345.6789", vec![Token::Float(12345.6789), Token::Eof]),
            (
                "1.1.1.1",
                vec![
                    Token::Error(String::from("Invalid float: 1.1.1.1")),
                    Token::Eof,
                ],
            ),
        ]
        .into_iter()
        .for_each(|(source, expected)| {
            let actual: Vec<Token> = Lexer::new(source).into_iter().collect();
            assert_eq!(actual, expected);
        });
    }

    #[test]
    fn test_strings() {
        vec![
            (
                "\"test1\"",
                vec![Token::String(String::from("test1")), Token::Eof],
            ),
            (
                "\"test1\" + \"test2\"",
                vec![
                    Token::String(String::from("test1")),
                    Token::Plus,
                    Token::String(String::from("test2")),
                    Token::Eof,
                ],
            ),
        ]
        .into_iter()
        .for_each(|(source, expected)| {
            let actual: Vec<Token> = Lexer::new(source).into_iter().collect();
            assert_eq!(actual, expected);
        });
    }

    #[test]
    fn test_source() {
        vec![(
            r###"
def test(x):
  if x > 0:
    return 1
  else:
    return 0
"###,
            vec![
                Token::NewLine,
                Token::Def,
                Token::Identifier(String::from("test")),
                Token::LeftParen,
                Token::Identifier(String::from("x")),
                Token::RightParen,
                Token::Colon,
                Token::NewLine,
                Token::Indent(2),
                Token::If,
                Token::Identifier(String::from("x")),
                Token::Greater,
                Token::Integer(0),
                Token::Colon,
                Token::NewLine,
                Token::Indent(4),
                Token::Return,
                Token::Integer(1),
                Token::NewLine,
                Token::Dedent(2),
                Token::Else,
                Token::Colon,
                Token::NewLine,
                Token::Indent(4),
                Token::Return,
                Token::Integer(0),
                Token::NewLine,
                Token::Eof,
            ],
        )]
        .into_iter()
        .for_each(|(source, expected)| {
            let actual: Vec<Token> = Lexer::new(source).into_iter().collect();
            assert_eq!(actual, expected);
        });
    }
}
