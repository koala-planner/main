use std::{collections::{HashSet, LinkedList, HashMap}, vec};
use std::rc::Rc;

use super::SearchNode;

use super::ComputeTree;
use super::ConnectionLabel;

#[derive(Debug)]
pub struct FMPolicy {
    pub transitions: Vec<(HashSet<u32>, Rc<Vec<String>>, String)>
}

impl FMPolicy {
    pub fn new(computation_history: &ComputeTree) -> FMPolicy {
        // vec of (state, vec(exectuted_task_names), new_task)
        let mut policy = vec![];
        let mut working_set: LinkedList<(u32, Rc<Vec<String>>)> = LinkedList::from([(computation_history.root, Rc::new(vec![]))]);
        // TOOD: for each branch the execution history changes
        while !working_set.is_empty() {
            let (id, history) = working_set.pop_front().unwrap();
            let node = computation_history.ids.get(&id).unwrap().borrow();
            // Is node terminal?
            match &node.connections {
                Some(connection) => {
                    let marked = connection.has_marked_connection().unwrap();
                    // Check whether transition is decomposition or primitive action execution
                    match marked.action_type {
                        ConnectionLabel::Decomposition => {
                            for child in marked.children.iter(){
                                working_set.push_back((*child, Rc::clone(&history)));
                            }
                        },
                        ConnectionLabel::Execution(t) => {
                            let task_name = node.search_node.tn.get_task(t).unwrap().get_name();
                            let state = node.search_node.state.as_ref().clone();
                            policy.push((state, history.clone(), task_name.clone()));
                            let mut new_history = history.as_ref().clone();
                            new_history.push(task_name);
                            let new_history = Rc::new(new_history);
                            for child in marked.children.iter() {
                                working_set.push_back((*child, Rc::clone(&new_history)));
                            }
                        }
                    }
                }
                None => { }
            } 
        }
        FMPolicy { transitions: policy }
    }
}