use crate::ast::Literal;
use crate::symbol_table::SymbolTable;
use crate::bytecode::Bytecode;

#[derive(Clone, Debug)]
pub struct Chunk {
    pub name: String,
    pub data: Vec<u8>,
    pub constants: Vec<Literal>,
    pub globals: SymbolTable,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            name: String::from("__main__"),
            data: Vec::new(),
            constants: Vec::new(),
            globals: SymbolTable::new(),
        }
    }

    pub fn get_data_u64(&self, index: usize) -> u64 {
        assert!(index + 8 < self.data.len());
        let bytes = &self.data[index..index + 8];
        if let Ok(array) = bytes.try_into() {
            return u64::from_ne_bytes(array);
        }
        unreachable!();
    }

    pub fn add_global(&mut self, identifier: &str) -> u64 {
        self.globals.insert(identifier, None)
    }

    pub fn add_constant(&mut self, literal: &Literal) -> u64 {
        self.constants.push(literal.clone());
        (self.constants.len() - 1) as u64
    }

    pub fn emit(&mut self, op: Bytecode) {
        self.data.push(op as u8);
    }

    pub fn emit_index(&mut self, index: u64) {
        self.data.extend_from_slice(&index.to_ne_bytes());
    }
}
