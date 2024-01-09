use crate::chunk::Chunk;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub arity: usize,
    pub chunk: Chunk,
    pub functions: HashMap<String, Box<Function>>,
}

impl PartialOrd for Function {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Function {
    pub fn new(name: String) -> Function {
        Function {
            name,
            arity: 0,
            chunk: Chunk::new(),
            functions: HashMap::new(),
        }
    }
}
