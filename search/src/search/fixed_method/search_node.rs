use std::{collections::{HashSet, BTreeSet, HashMap}, rc::Rc, cell::RefCell};
use crate::{task_network::Applicability, relaxation::ToClassical};
use crate::heuristic_calculator::FF;
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
    
    pub fn compute_heuristic_value(&self, encoder: &ToClassical, bijection: &HashMap<u32, u32>) -> f32 {
        let task_ids = self.tn.get_all_task_mappings().iter().map(|x| {
            *bijection.get(x).unwrap()
        }).collect();
        let relaxed_state = encoder.compute_relaxed_state(
            &task_ids,
            self.state.as_ref()
        );
        let goal_state = encoder.compute_goal_state(&task_ids);
        let ff_val = FF::calculate_h(&encoder.domain, &relaxed_state, &goal_state);
        if ff_val > (task_ids.len() as f32) { ff_val } else { task_ids.len() as f32 }
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
            expansions.extend(*abstract_expansions);
        }
        expansions
    }

    fn expand_primitives(&self, primitive_tasks: BTreeSet<u32>) -> Vec<NodeExpansion> {
        let mut expansion = vec![];
        for t in primitive_tasks.iter() {
            if let Task::Primitive(a) = &*self.tn.get_task(*t).borrow() {
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
            };
        }
        expansion
    } 

    fn expand_abstract_task(&self, task_id: u32) -> Box<Vec<NodeExpansion>> {
        if let Task::Compound(t) = &*self.tn.get_task(task_id).borrow() {
            let mut expansions = vec![];
            for method in t.methods.iter() {
                let new_tn = self.tn.decompose(task_id, method);
                let new_search_node = SearchNode::new(
                    Rc::clone(&self.state),
                    Rc::new(new_tn),
                );
                expansions.push(
                    NodeExpansion {
                        connection_label: ConnectionLabel::Decomposition(method.name.clone()),
                        items: vec![new_search_node]
                    });
            }
            return Box::new(expansions)
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

    pub fn get_label(&self) -> String {
        match self {
            Self::Execution(name, _) => name.clone(),
            Self::Decomposition(name) => name.clone()
        }
    }
}

impl PartialEq for ConnectionLabel {
    fn eq(&self, rhs: &ConnectionLabel) -> bool {
        match self {
            Self::Execution(name, _) => {
                if let ConnectionLabel::Execution(name_rhs, _) = rhs {
                    return name == name_rhs;
                } else {
                    false
                }
            },
            Self::Decomposition(name) => {
                if let ConnectionLabel::Decomposition(name_rhs) = rhs {
                    return name == name_rhs;
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::{task_network::Method, domain_description::DomainTasks};
    use super::*;

    #[test]
    pub fn expansion_correctness_test() {
        let p1 = Task::Primitive(PrimitiveAction::new(
            "p1".to_string(), 
            1, 
            HashSet::from([0]), 
            vec![HashSet::from([1]), HashSet::from([2,4])],
            vec![HashSet::from([3]), HashSet::new()] 
        ));
        let p2 = Task::Primitive(PrimitiveAction::new(
            "p2".to_string(), 
            1, 
            HashSet::from([1, 2, 4]), 
            vec![HashSet::from([1]),],
            vec![HashSet::from([3]),] 
        ));
        let p3 = Task::Primitive(PrimitiveAction::new(
            "p3".to_string(), 
            1, 
            HashSet::from([0, 3]), 
            vec![HashSet::from([1]),],
            vec![HashSet::from([3]),] 
        ));
        let p4 = Task::Primitive(PrimitiveAction::new(
            "p4".to_string(), 
            1, 
            HashSet::from([4]), 
            vec![HashSet::from([2]),],
            vec![HashSet::new(),] 
        ));
        let t1 = Task::Compound(CompoundTask::new(
            "t1".to_string(),
            vec![]
        ));
        let domain = Rc::new(DomainTasks::new(vec![p1, p2, p3, p4, t1]));
        let m1 = Method::new(
            "m1".to_string(),
            HTN::new(
                BTreeSet::from([1,2]),
                vec![(1,2)],
                domain.clone(),
                HashMap::from([(1, domain.get_id("p1")), (2, domain.get_id("p2"))]))
        );
        let m2 = Method::new(
            "m2".to_string(),
            HTN::new(
                BTreeSet::from([1,2]),
                vec![], 
                domain.clone(),
                HashMap::from([(1, domain.get_id("p3")), (2, domain.get_id("p4"))]))
        );
        let id = domain.get_id("t1");
        let domain = domain.add_methods(vec![(id, m1), (id, m2)]);
        let tn = HTN::new(
            BTreeSet::from([1, 2, 3, 4]), 
            vec![(1,4), (2,4), (3,4)],
            domain.clone(),
            HashMap::from([(1, domain.get_id("p1")), (2, domain.get_id("t1")),
            (3, domain.get_id("p3")), (4, domain.get_id("p4"))])
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