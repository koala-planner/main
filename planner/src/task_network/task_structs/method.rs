use std::hash::Hash;
use core::{hash::Hasher, fmt};

use crate::task_network::network::HTN;

#[derive(Debug, Clone)]
pub struct Method{
    pub name: String,
    pub decomposition: HTN,
}

impl Method {
    pub fn new(name: String, decomposition: HTN) -> Method {
        Method {
            name: name,
            decomposition: decomposition,
        }
    }
}