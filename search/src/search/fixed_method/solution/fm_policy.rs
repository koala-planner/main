use std::{collections::{HashSet, LinkedList, HashMap}, vec};
use std::rc::Rc;

use crate::domain_description::{DomainTasks, Facts};

use super::SearchNode;

use super::ComputeTree;
use super::ConnectionLabel;

#[derive(Debug)]
pub struct FMPolicyNode{
    pub state: HashSet<String>,
    pub execution_history: Rc<Vec<String>>
}

#[derive(Debug)]
pub struct FMPolicy {
    pub transitions: Vec<(FMPolicyNode, String)>
}

impl FMPolicy {
    pub fn new(facts: &Facts, computation_history: &ComputeTree) -> FMPolicy {
        // vec of (state, vec(exectuted_task_names), new_task)
        let mut policy = vec![];
        let mut working_set: LinkedList<(u32, Rc<Vec<String>>)> = LinkedList::from([(computation_history.root, Rc::new(vec![]))]);
        // TOOD: for each branch the execution history changes
        while !working_set.is_empty() {
            let (id, history) = working_set.pop_front().unwrap();
            let node = computation_history.ids.get(&id).unwrap().borrow();
            let state: HashSet<String> = node.search_node.state.as_ref().iter().map(|x| {
                facts.get_fact(*x).clone()
            }).collect();
            // Is node terminal?
            match &node.connections {
                Some(connection) => {
                    if let Some(marked) = connection.has_marked_connection() {
                        // Check whether transition is decomposition or primitive action execution
                        match &marked.action_type {
                            ConnectionLabel::Decomposition(_) => {
                                for child in marked.children.iter(){
                                    working_set.push_back((*child, Rc::clone(&history)));
                                }
                            },
                            ConnectionLabel::Execution(name, cost) => {
                                let new_policy_node = FMPolicyNode {
                                    state: state,
                                    execution_history: history.clone()
                                };
                                policy.push((new_policy_node, name.clone()));
                                let mut new_history = history.as_ref().clone();
                                new_history.push(name.clone());
                                let new_history = Rc::new(new_history);
                                for child in marked.children.iter() {
                                    working_set.push_back((*child, Rc::clone(&new_history)));
                                }
                            }
                        }
                    }
                    else {
                        unreachable!()
                    }
                }
                None => {
                    let new_policy_node = FMPolicyNode {
                        state: state,
                        execution_history: history.clone()
                    };
                    policy.push((new_policy_node, "None".to_string()));
                }
            } 
        }
        FMPolicy { transitions: policy }
    }
}

impl std::fmt::Display for FMPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for (policy_node, action) in self.transitions.iter() {
            writeln!(f, "State: {:?}\nHistory: {:?}\nAction: {}", policy_node.state, policy_node.execution_history, action);
            writeln!(f, "---------------------------------------------");
        }
        Ok(())
    }
}