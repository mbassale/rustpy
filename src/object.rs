use crate::ast::Literal;

#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    None,
    True,
    False,
    Integer(i64),
    Float(f64),
    String(String),
}

impl Object {
    pub fn from_literal(literal: &Literal) -> Object {
        match literal {
            Literal::None => Object::None,
            Literal::True => Object::True,
            Literal::False => Object::False,
            Literal::Integer(value) => Object::Integer(*value),
            Literal::Float(value) => Object::Float(*value),
            Literal::String(value) => Object::String(value.to_string()),
        }
    }

    pub fn is_none(&self) -> bool {
        self == &Object::None
    }

    pub fn is_true(&self) -> bool {
        self == &Object::True
    }

    pub fn is_false(&self) -> bool {
        self == &Object::False
    }

    pub fn is_integer(&self) -> bool {
        match self {
            Object::Integer(_) => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Object::Float(_) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Object::String(_) => true,
            _ => false,
        }
    }
}
