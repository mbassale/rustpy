use crate::ast::Literal;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Object {
    pub id: u64,
    pub name: String,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    None,
    True,
    False,
    Integer(i64),
    Float(f64),
    String(String),
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::None => (),
            Self::True => (),
            Self::False => (),
            Self::Float(value) => {
                state.write_u128(*value as u128);
            }
            Self::Integer(value) => value.hash(state),
            Self::String(value) => value.hash(state),
        }
    }
}

impl Object {
    pub fn new_with_id(id: u64, name: String, value: Value) -> Object {
        Object { id, name, value }
    }

    pub fn new(value: Value) -> Object {
        let mut object = Object {
            id: 0,
            name: String::new(),
            value: value.clone(),
        };
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        object.id = hasher.finish();
        object.name = String::from("$") + &object.id.to_string();
        object
    }

    pub fn new_none() -> Object {
        Object {
            id: 0,
            name: String::from("None"),
            value: Value::None,
        }
    }

    pub fn new_true() -> Object {
        Object {
            id: 1,
            name: String::from("True"),
            value: Value::True,
        }
    }

    pub fn new_false() -> Object {
        Object {
            id: 2,
            name: String::from("False"),
            value: Value::False,
        }
    }

    pub fn from_literal(literal: &Literal) -> Object {
        match literal {
            Literal::None => Object::new_none(),
            Literal::True => Object::new_true(),
            Literal::False => Object::new_false(),
            Literal::Integer(value) => Object::new(Value::Integer(*value)),
            Literal::Float(value) => Object::new(Value::Float(*value)),
            Literal::String(value) => Object::new(Value::String(value.to_string())),
        }
    }

    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn is_none(&self) -> bool {
        self.value == Value::None
    }

    pub fn is_true(&self) -> bool {
        self.value == Value::True
    }

    pub fn is_false(&self) -> bool {
        self.value == Value::False
    }

    pub fn is_integer(&self) -> bool {
        match self.value {
            Value::Integer(_) => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self.value {
            Value::Float(_) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self.value {
            Value::String(_) => true,
            _ => false,
        }
    }
}
