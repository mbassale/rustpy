use crate::object::{Object, Value};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SymbolTable {
    data: HashMap<u64, Object>,
    last_idx: u64,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            data: HashMap::new(),
            last_idx: 0,
        }
    }

    pub fn get(&self, id: u64) -> Option<&Object> {
        self.data.get(&id)
    }

    pub fn set(&mut self, id: u64, obj: Object) {
        self.data.insert(id, obj);
    }

    pub fn insert(&mut self, name: &str, obj: Option<Object>) -> u64 {
        self.last_idx += 1;
        match obj {
            Some(mut obj) => {
                obj.id = self.last_idx;
                self.data.insert(self.last_idx, obj);
            }
            None => {
                let obj = Object::new_with_id(self.last_idx, name.to_string(), Value::None);
                self.data.insert(self.last_idx, obj);
            }
        };
        self.last_idx
    }

    pub fn extend(&mut self, symbol_table: SymbolTable) {
        self.data.extend(symbol_table.data);
    }

    pub fn contains(&self, id: u64) -> bool {
        self.data.contains_key(&id)
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.data.iter().any(|(_, v)| v.name == name)
    }

    pub fn get_index(&self, name: &str) -> u64 {
        for (idx, obj) in &self.data {
            if obj.name == name {
                return *idx;
            }
        }
        unreachable!();
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}
