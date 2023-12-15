use std::hash::Hash;

use super::Method;

#[derive(Debug)]
#[derive(Clone)]
pub struct CompoundTask{
    pub name: String,
    pub methods: Vec<Method>,
}

impl CompoundTask {
    pub fn new(name: String, methods: Vec<Method>) -> Self {
        CompoundTask { name, methods }
    }

    pub fn add_method(&mut self, method: Method) {
        self.methods.push(method);
    }
}
