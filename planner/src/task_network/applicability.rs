use std::collections::HashSet;

pub trait Applicability {
    fn is_applicable(&self, state: &HashSet<u32>) -> bool;
    // non-determinstic transition function
    fn transition(&self, state: &HashSet<u32>) -> Vec<HashSet<u32>>;
}
