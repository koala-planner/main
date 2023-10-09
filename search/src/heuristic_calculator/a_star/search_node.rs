use std::{rc::Rc, collections::HashSet};

pub struct SearchNode{
    pub g: f32,
    pub h: f32,
    pub state: Rc<HashSet<u32>>,
    pub hist: Vec<String>
}

impl PartialEq for SearchNode {
    fn eq(&self, rhs: &SearchNode) -> bool {
        (self.g + self.h) == (rhs.g + rhs.g)
    }
}

impl Eq for SearchNode {}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, rhs: &SearchNode) -> Option<std::cmp::Ordering> {
        let lhs_val = self.g + self.h;
        let rhs_val = rhs.g + rhs.h;
        match lhs_val.partial_cmp(&rhs_val) {
            Some(x) => Some(x.reverse()),
            None => None
        }
    }
}

impl Ord for SearchNode {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        match self.partial_cmp(rhs) {
            Some(ordering) => ordering,
            None => panic!("Undefined Comparison")
        }
    }
}
