use crate::ast::Literal;
use crate::bytecode::Bytecode;
use crate::function::Function;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Chunk {
    pub name: String,
    pub data: Vec<u8>,
    pub constants: Vec<Literal>,
    pub functions: Vec<Box<Function>>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            name: String::from("__main__"),
            data: Vec::new(),
            constants: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn size(&self) -> u64 {
        self.data.len() as u64
    }

    pub fn get_data_u64_safe(&self, index: usize) -> Option<u64> {
        match self.data.get(index..index + 8) {
            Some(bytes) => {
                if let Ok(array) = bytes.try_into() {
                    return Some(u64::from_ne_bytes(array));
                }
                return None;
            }
            None => None,
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

    pub fn add_constant(&mut self, literal: &Literal) -> u64 {
        self.constants.push(literal.clone());
        (self.constants.len() - 1) as u64
    }

    pub fn emit(&mut self, op: Bytecode) {
        self.data.push(op as u8);
    }

    pub fn emit_index(&mut self, index: u64) -> u64 {
        let index_addr = self.size();
        self.data.extend_from_slice(&index.to_ne_bytes());
        index_addr
    }

    pub fn patch_jump_addr(&mut self, jump_offset_addr: u64, target_addr: u64) {
        let offset = target_addr - jump_offset_addr + 1;
        let index_bytes = offset.to_ne_bytes();
        for i in 0..index_bytes.len() {
            self.data[jump_offset_addr as usize + i] = index_bytes[i];
        }
    }
}
