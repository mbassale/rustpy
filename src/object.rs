use crate::ast::Literal;
use crate::function::Function;
use crate::native::NativeFunction;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq, PartialOrd, Hash)]
pub struct Object {
    pub id: u64,
    pub name: String,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Value {
    None,
    True,
    False,
    Integer(i64),
    Float(f64),
    String(String),
    Function(Function),
    NativeFunction(NativeFunction),
}

impl Value {
    pub fn new_from_bool(value: bool) -> Value {
        if value {
            Value::True
        } else {
            Value::False
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::None => false,
            Value::True => true,
            Value::False => false,
            Value::Integer(value) => *value != 0,
            Value::Float(value) => *value != 0.0,
            Value::String(value) => !value.is_empty(),
            Value::Function(_) => true,
            Value::NativeFunction(_) => true,
        }
    }

    pub fn is_falsey(&self) -> bool {
        match self {
            Value::None => true,
            Value::True => false,
            Value::False => true,
            Value::Integer(value) => *value == 0,
            Value::Float(value) => *value == 0.0,
            Value::String(value) => value.is_empty(),
            Value::Function(_) => false,
            Value::NativeFunction(_) => false,
        }
    }

    pub fn is_integer(&self) -> bool {
        match self {
            Value::Integer(_) => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Value::Float(_) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Value::String(_) => true,
            _ => false,
        }
    }

    pub fn is_callable(&self) -> bool {
        match self {
            Value::Function(_) => true,
            Value::NativeFunction(_) => true,
            _ => false,
        }
    }
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
            Self::Function(function) => function.name.hash(state),
            Self::NativeFunction(function) => function.name.hash(state),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::True => write!(f, "True"),
            Self::False => write!(f, "False"),
            Self::Integer(value) => write!(f, "{}", value),
            Self::Float(value) => write!(f, "{}", value),
            Self::String(value) => write!(f, "{}", value),
            Self::Function(function) => {
                write!(f, "<function:{}:{}>", function.name, function.arity)
            }
            Self::NativeFunction(function) => {
                write!(f, "<native:{}:{}>", function.name, function.arity)
            }
        }
    }
}

impl Object {
    pub fn new_with_id(id: u64, name: String, value: Value) -> Object {
        Object { id, name, value }
    }

    pub fn new_with_name(name: String, value: Value) -> Object {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        Object {
            id: hasher.finish(),
            name,
            value,
        }
    }

    pub fn new(value: Value) -> Object {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let mut object = Object {
            id: 0,
            name: String::new(),
            value,
        };
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

    pub fn is_truthy(&self) -> bool {
        self.value.is_truthy()
    }

    pub fn is_falsey(&self) -> bool {
        self.value.is_falsey()
    }

    pub fn is_integer(&self) -> bool {
        self.value.is_integer()
    }

    pub fn is_float(&self) -> bool {
        self.value.is_float()
    }

    pub fn is_string(&self) -> bool {
        self.value.is_string()
    }

    pub fn is_callable(&self) -> bool {
        self.value.is_callable()
    }
}
