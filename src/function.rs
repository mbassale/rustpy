use crate::chunk::Chunk;

const GLOBAL_SCOPE: &str = "<main>";

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
    pub fn new_global_scope() -> Function {
        Function {
            name: GLOBAL_SCOPE.to_string(),
            arity: 0,
            chunk: Chunk::new(),
        }
    }
    pub fn new(name: String) -> Function {
        Function {
            name,
            arity: 0,
            chunk: Chunk::new(),
        }
    }

    pub fn is_global_scope(&self) -> bool {
        self.name == GLOBAL_SCOPE
    }
}
