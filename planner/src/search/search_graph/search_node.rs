use std::{collections::{HashSet, HashMap}, rc::Rc};

use super::*;
use super::{HTN, PrimitiveAction, Task, CompoundTask, h_type, HeuristicType};

use crate::{task_network::Applicability, relaxation::RelaxedComposition, heuristics::*};

#[derive(Debug)]
pub struct SearchGraphNode {
    pub parents: Option<Vec<u32>>,
    pub tn: Rc<HTN>,
    pub state: Rc<HashSet<u32>>,
    pub connections: Option<NodeConnections>,
    pub cost: f32,
    pub status: NodeStatus,
    pub depth: u16,
}

#[derive(Debug, Clone)]
pub enum NodeStatus {
    Solved,
    OnGoing,
    Failed
}

impl NodeStatus {
    pub fn is_terminal(&self) -> bool {
        match self {
            Self::Failed => true,
            Self::Solved => true,
            Self::OnGoing => false
        }
    }
}

impl SearchGraphNode {
    pub fn mark(&mut self, i: u32) {
        self.clear_marks();
        self.connections.as_mut().unwrap().mark(i)
    }
    pub fn get_marked_connection(&self) -> Option<&Connector> {
        for item in self.connections.as_ref().unwrap().children.iter() {
            if item.is_marked {
                return Some(item);
            }
        }
        None
    }
    pub fn clear_marks(&mut self) {
        self.connections.as_mut().unwrap().clear_marks()
    }

    pub fn has_children(&self) -> bool {
        match self.connections {
            Some(_) => true,
            None => false
        }
    }

    pub fn add_parent(&mut self, id: u32) {
        self.parents = match &self.parents {
            Some(parents) => {
                let p = parents.clone();
                Some(p)
            },
            None => {
                panic!("attempting to add parent to root");
            }
        }
    } 

    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }

    pub fn is_goal(&self) -> bool {
        self.tn.is_empty()
    }

    pub fn h_val(tn: &HTN, state: &HashSet<u32>, encoder: &RelaxedComposition, bijection: &HashMap<u32, u32>, h_type: &HeuristicType) -> f32 {
        let occurances = tn.count_tasks_with_frequency();
        let task_ids = occurances.iter().map(|(task, _)| {
            *bijection.get(task).unwrap()
        }).collect();
        let relaxed_state = encoder.compute_relaxed_state(
            &task_ids,
            state
        );
        let goal_state = encoder.compute_goal_state(&task_ids);
        let mut val = match h_type {
            HeuristicType::HFF => h_ff(&encoder.domain, &relaxed_state, &goal_state),
            HeuristicType::HAdd => h_add(&encoder.domain, &relaxed_state, &goal_state),
            HeuristicType::HMax => h_max(&encoder.domain, &relaxed_state, &goal_state),
        };
        
        // Compensate for the repetition of tasks
        for (_, count) in occurances {
            if count > 1 {
                val += (count - 1) as f32
            }
        }
        val
    }
}