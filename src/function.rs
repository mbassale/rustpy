use crate::chunk::Chunk;

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub arity: usize,
    pub chunk: Chunk,
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
        }
    }
}
