use std::{collections::{HashSet, LinkedList, HashMap}, vec};
use std::rc::Rc;

use crate::{domain_description::{DomainTasks, Facts}, task_network::HTN};

use super::*;


#[derive(Debug)]
pub struct PolicyOutput{
    pub task: String,
    pub method: String 
}

#[derive(Debug)]
pub struct StrongPolicy {
    pub transitions: Vec<(PolicyNode, PolicyOutput)>,
    pub makespan: u16,
}

impl StrongPolicy {
    pub fn new(facts: &Facts, computation_history: &SearchGraph) -> StrongPolicy {
        // vec of (state, vec(exectuted_task_names), new_task)
        let mut policy = vec![];
        let mut visited = HashSet::new();
        let mut working_set: LinkedList<u32> = LinkedList::from([computation_history.root]);
        let mut makespan = u16::MIN;;
        // TOOD: for each branch the execution history changes
        while !working_set.is_empty() {
            let id = working_set.pop_front().unwrap();
            if visited.contains(&id) {
                continue;
            } else {
                visited.insert(id);
            }
            let node = computation_history.ids.get(&id).unwrap().borrow();
            let state: HashSet<String> = node.search_node.state.as_ref().iter().map(|x| {
                facts.get_fact(*x).clone()
            }).collect();
            let input = PolicyNode {
                tn: node.search_node.tn.clone(),
                state: state
            };
            // Is node terminal?
            match &node.connections {
                Some(connection) => {
                    if let Some(marked) = connection.has_marked_connection() {
                        if node.depth > makespan {
                            makespan = node.depth
                        }
                        // Check whether transition is decomposition or primitive action execution
                        match &marked.action_type {
                            ConnectionLabel::Decomposition(name, method) => {
                                let output = PolicyOutput {
                                    task: name.clone(),
                                    method: method.clone()
                                };
                                policy.push((input, output));
                                for child in marked.children.iter(){
                                    working_set.push_back(*child);
                                }
                            },
                            ConnectionLabel::Execution(name, _) => {
                                let output = PolicyOutput {
                                    task: name.clone(),
                                    method: "ε".to_string()
                                };
                                policy.push((input, output));
                                for child in marked.children.iter() {
                                    working_set.push_back(*child);
                                }
                            }
                        }
                    }
                    else {
                        unreachable!()
                    }
                }
                None => {
                    
                }
            } 
        }
        StrongPolicy { transitions: policy, makespan: makespan }
    }
}

impl std::fmt::Display for StrongPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for (input, output) in self.transitions.iter() {
            writeln!(f, "TN: {} \nState: {:?}\nTask: {}\nMethod: {}", input.tn, input.state, output.task, output.method);
            writeln!(f, "---------------------------------------------");
        }
        Ok(())
    }
}