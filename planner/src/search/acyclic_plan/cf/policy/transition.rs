use super::PolicyNode;
pub struct PolicyTransition<'a> {
    n1: &'a PolicyNode,
    task: (String, String),
    n2: &'a PolicyNode
}