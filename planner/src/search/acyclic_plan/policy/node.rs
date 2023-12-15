use std::collections::HashSet;
use std::rc::Rc;
use super::HTN;
#[derive(Debug)]
pub struct PolicyNode{
    pub state: HashSet<String>,
    pub tn: Rc<HTN>
}