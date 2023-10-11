use std::collections::HashSet;

use crate::task_network::PrimitiveAction;

use super::Facts;

#[derive(Debug)]
pub struct ClassicalDomain {
    pub facts: Facts,
    pub actions: Vec<PrimitiveAction>,
}

impl ClassicalDomain {
    pub fn new(
        facts: Facts,
        actions: Vec<PrimitiveAction>,
    ) -> ClassicalDomain {
        ClassicalDomain { facts, actions}
    }

    pub fn delete_relax(&self) -> ClassicalDomain {
        let new_actions = self.actions.iter()
            .map(|a| {
                a.delete_relax()
            }).collect();
        ClassicalDomain::new(
            self.facts.clone(),
            new_actions,
        )
    }

    pub fn get_actions_by_index(&self, indices: HashSet<usize>) -> Vec<&PrimitiveAction> {
        self.actions.iter().enumerate().filter(|(i, action)| {
            indices.contains(i)
        }).map(|(i, action)| action).collect()
    }
}