use std::{collections::HashSet, rc::Rc, cell::RefCell};
use crate::{task_network::Applicability, relaxation::ToClassical, heuristic_calculator::FF};

use super::{HTN, PrimitiveAction, Task, CompoundTask};
#[derive(Debug, Clone)]
pub struct SearchNode{
    pub state: Rc<HashSet<u32>>,
    pub tn: Rc<HTN>,
}

impl SearchNode {
    pub fn new(state: Rc<HashSet<u32>>,tn: Rc<HTN>) -> SearchNode {
        SearchNode { state, tn }
    }
    
    pub fn compute_heuristic_value(&self, encoder: &ToClassical) -> f32 {
        // let relaxed_state = encoder.compute_relaxed_state(&self.tn, self.state.as_ref());
        // let goal_state = encoder.compute_goal_state(&self.tn);
        // FF::calculate_h(&encoder.domain, &relaxed_state, &goal_state)
        0.0
    }

    pub fn is_goal(&self) -> bool {
        self.tn.is_empty()
    }

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
            let abstract_expansions = self.expand_abstract_task(*abstract_id);
            expansions.extend(abstract_expansions);
        }
        expansions
    }

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
                            connection_label: ConnectionLabel::Execution(a.name.clone(), a.cost),
                            items: vec![new_search_node],
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
                            connection_label: ConnectionLabel::Execution(a.name.clone(), a.cost),
                            items: new_search_nodes,
                        });
                    }
                }
            }
        }
        expansion
    } 

    fn expand_abstract_task(&self, task_id: u32) -> Vec<NodeExpansion> {
        if let Task::Compound(t) = self.tn.get_task(task_id).unwrap().as_ref() {
            let mut expansions = vec![];
            for method in t.methods.iter() {
                let new_tn = self.tn.decompose(task_id, method);
                let new_search_node = SearchNode::new(
                    Rc::clone(&self.state),
                    Rc::new(new_tn),
                );
                let expansion = NodeExpansion {
                    connection_label: ConnectionLabel::Decomposition(t.name.clone()),
                    items: vec![new_search_node],
                };
                expansions.push(expansion);
            }
            return expansions
        }   
        unreachable!()     
    }
}


#[derive(Debug)]
pub struct NodeExpansion {
    pub connection_label: ConnectionLabel,
    pub items: Vec<SearchNode>,
}

#[derive(Debug)]
pub enum ConnectionLabel {
    Execution(String, u32), 
    Decomposition(String)
}

impl ConnectionLabel {
    pub fn is_decomposition(&self) -> bool {
        match  &self {
            ConnectionLabel::Decomposition(_) => true,
            _ => false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::task_network::Method;
    use super::*;

    #[test]
    pub fn expansion_correctness_test() {
        let p1 = Rc::new(Task::Primitive(PrimitiveAction::new(
            "p1".to_string(), 
            1, 
            HashSet::from([0]), 
            vec![HashSet::from([1]), HashSet::from([2,4])],
            vec![HashSet::from([3]), HashSet::new()] 
        )));
        let p2 = Rc::new(Task::Primitive(PrimitiveAction::new(
            "p2".to_string(), 
            1, 
            HashSet::from([1, 2, 4]), 
            vec![HashSet::from([1]),],
            vec![HashSet::from([3]),] 
        )));
        let p3 = Rc::new(Task::Primitive(PrimitiveAction::new(
            "p3".to_string(), 
            1, 
            HashSet::from([0, 3]), 
            vec![HashSet::from([1]),],
            vec![HashSet::from([3]),] 
        )));
        let p4 = Rc::new(Task::Primitive(PrimitiveAction::new(
            "p4".to_string(), 
            1, 
            HashSet::from([4]), 
            vec![HashSet::from([2]),],
            vec![HashSet::new(),] 
        )));
        let m1 = Method::new(
            "m1".to_string(),
            HTN::new(
                HashSet::from([1,2]), vec![(1,2)], HashMap::from([(1, p1.clone()), (2, p2.clone())]))
        );
        let m2 = Method::new(
            "m2".to_string(),
            HTN::new(
                HashSet::from([1,2]), vec![], HashMap::from([(1, p3.clone()), (2, p4.clone())]))
        );
        let t1 = Rc::new(Task::Compound(CompoundTask::new(
            "t1".to_string(),
            vec![m1, m2]
        )));

        let tn = HTN::new(
            HashSet::from([1, 2, 3, 4]), 
            vec![(1,4), (2,4), (3,4)],
            HashMap::from([
                (1, p1), (2, t1), (3, p3), (4, p4)
            ])
        );
        let state = HashSet::from([0,3]);
        let sn = SearchNode::new(Rc::new(state), Rc::new(tn));
        let expansion = sn.expand();
        assert_eq!(expansion.len(), 4);
        let exp_p1: Vec<&NodeExpansion> = expansion.iter().filter(|x| {
            match &x.connection_label {
                ConnectionLabel::Execution(x, 1) if x == "p1" => true,
                _ => false
            }
        }).collect();
        assert_eq!(exp_p1.len(), 1);
        assert_eq!(exp_p1[0].items.len(), 2);
        for node in exp_p1[0].items.iter() {
            assert_eq!(node.tn.count_tasks(), 3);
        }
        let exp_t1: Vec<&NodeExpansion> = expansion.iter().filter(|x| {
            match x.connection_label {
                ConnectionLabel::Decomposition(_) => true,
                _ => false
            }
        }).collect();
        assert_eq!(exp_t1.len(), 2);
    }
}