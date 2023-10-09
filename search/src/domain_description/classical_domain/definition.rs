use std::collections::HashSet;

use crate::task_network::PrimitiveAction;

use super::Facts;

#[derive(Debug)]
pub struct ClassicalDomain {
    pub facts: Facts,
    pub actions: Vec<PrimitiveAction>,
    pub init_state: HashSet<u32>,
    pub goal_state: HashSet<u32>
}

impl ClassicalDomain {
    pub fn new(
        facts: Facts,
        actions: Vec<PrimitiveAction>,
        init_state: HashSet<u32>,
        goal_state: HashSet<u32>
    ) -> ClassicalDomain {
        ClassicalDomain { facts, actions, init_state, goal_state }
    }

    pub fn delete_relax(&self) -> ClassicalDomain {
        let new_actions = self.actions.iter()
            .map(|a| {
                a.delete_relax()
            }).collect();
        ClassicalDomain::new(
            self.facts.clone(),
            new_actions, 
            self.init_state.clone(), 
            self.goal_state.clone()
        )
    }
}