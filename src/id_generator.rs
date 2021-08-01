use std::{cell::RefCell};

pub struct IdGenerator {
    id: i32,
    count: i32,
}

impl IdGenerator {
    fn new() -> Self {
        Self {
            id: -1,
            count: 0
        }
    }
    fn new_id(&mut self) -> i32 {
        self.id + 1
    }
    pub fn get(&mut self) -> i32 {
        let new_id = self.new_id();
        self.count += 1;
        self.id = new_id;
        new_id
    }
    pub fn count(&self) -> i32 {
        self.count
    }
}

thread_local!(pub static ID_GENERATOR: RefCell<IdGenerator> = RefCell::new(IdGenerator::new()));
