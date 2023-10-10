use std::{collections::HashSet, rc::Rc, cell::RefCell};
use crate::{task_network::Applicability, relaxation::ToClassical};

use super::{HTN, PrimitiveAction, Task, CompoundTask};
#[derive(Debug)]
pub struct SearchNode{
    pub state: Rc<HashSet<u32>>,
    pub tn: Rc<HTN>,
}

impl SearchNode {
    pub fn new(state: Rc<HashSet<u32>>,tn: Rc<HTN>) -> SearchNode {
        SearchNode { state, tn }
    }
    
    pub fn compute_heuristic_value(&self, encoder: &ToClassical) -> f32 {
        let relaxed_state = encoder.compute_relaxed_state(&self.tn, self.state.as_ref());
        // TODO: implement FF
        todo!()
    }

    pub fn is_goal(&self) -> bool {
        self.tn.is_empty()
    }

    // TODO: test
    pub fn expand(&self) -> Vec<NodeExpansion> {
        if self.is_goal() {
            return vec![];
        }
        let tn = &self.tn;
        let unconstrained = tn.get_unconstrained_tasks();
        let (abstract_tasks, primitive_tasks) = tn.separate_tasks(&unconstrained);
        // expand all primitives
        let mut expansions = self.expand_primitives(primitive_tasks);
        // expand all abstract tasks
        for abstract_id in abstract_tasks.iter() {
            let search_node = self.expand_abstract_task(*abstract_id);
            expansions.push(NodeExpansion {
                connection_label: ConnectionLabel::Decomposition,
                items: search_node, connector: Connector::OR
            })
        }
        expansions
    }

    // TODO: test
    fn expand_primitives(
        &self,
        primitive_tasks: HashSet<u32>
    )-> Vec<NodeExpansion> {
        let mut expansion = vec![];
        for t in primitive_tasks.iter() {
            if let Task::Primitive(a) = self.tn.get_task(*t).unwrap().as_ref() {
                if a.is_applicable(&self.state) {
                    if a.is_deterministic() {
                        let new_tn = self.tn.apply_action(*t);
                        let new_state = a.transition(&self.state)[0].clone();
                        let new_search_node = SearchNode::new(
                            Rc::new(new_state),
                            Rc::new(new_tn)
                        );
                        expansion.push(NodeExpansion {
                            connection_label: ConnectionLabel::Execution(*t),
                            items: vec![new_search_node], connector: Connector::OR
                        });
                    } else {
                        let new_tn = Rc::new(self.tn.apply_action(*t));
                        let new_states = a.transition(&self.state);
                        let mut new_search_nodes = vec![];
                        for nd_state in new_states.into_iter() {
                            let new_search_node = SearchNode::new(
                                Rc::new(nd_state),
                                Rc::clone(&new_tn)
                            );
                            new_search_nodes.push(new_search_node);
                        }
                        expansion.push(NodeExpansion {
                            connection_label: ConnectionLabel::Execution(*t),
                            items: new_search_nodes,
                            connector: Connector::AND
                        });
                    }
                }
            }
        }
        expansion
    } 

    // TODO: test
    fn expand_abstract_task(&self, task_id: u32) -> Vec<SearchNode> {
        if let Task::Compound(t) = self.tn.get_task(task_id).unwrap().as_ref() {
            let mut new_search_nodes = vec![];
            for method in t.methods.iter() {
                let new_tn = self.tn.decompose(task_id, method);
                let new_search_node = SearchNode::new(
                    Rc::clone(&self.state),
                    Rc::new(new_tn),
                );
                new_search_nodes.push(new_search_node);
            }
            return new_search_nodes
        }   
        unreachable!()     
    } 
}

#[derive(Debug)]
pub enum Connector {
    AND,
    OR
}

#[derive(Debug)]
pub struct NodeExpansion {
    pub connection_label: ConnectionLabel,
    pub items: Vec<SearchNode>,
    pub connector: Connector
}

#[derive(Debug)]
pub enum ConnectionLabel {
    Execution(u32), 
    Decomposition
}