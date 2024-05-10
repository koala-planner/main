use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap, HashSet},
    rc::Rc,
};
use super::*;

pub fn progress(tn: Rc<HTN>, state: Rc<HashSet<u32>>) -> Vec<NodeExpansion> {
    if tn.is_goal() {
        return vec![];
    }
    let unconstrained = tn.get_unconstrained_tasks();
    let (abstract_tasks, primitive_tasks) = tn.separate_tasks(&unconstrained);
    let mut expansions = vec![];
    // expand all primitives
    for p in primitive_tasks.iter() {
        if let Task::Primitive(a) = &*tn.get_task(*p).borrow() {
            if a.is_applicable(state.as_ref()) {
                if a.is_deterministic() {
                    let new_tn = tn.apply_action(*p);
                    let new_state = a.transition(state.as_ref())[0].clone();
                    expansions.push(NodeExpansion {
                        connection_label: ConnectionLabel::Execution(a.name.clone(), a.cost),
                        tn: Rc::new(new_tn),
                        states: vec![Rc::new(new_state)]
                    });
                } else {
                    let new_tn = Rc::new(tn.apply_action(*p));
                    let new_states = a.transition(state.as_ref());
                    let new_states = new_states
                        .into_iter()
                        .map(|x| {
                            Rc::new(x)
                        }).collect();
                    expansions.push(NodeExpansion {
                        connection_label: ConnectionLabel::Execution(a.name.clone(), a.cost),
                        tn: new_tn,
                        states: new_states
                    });
                }
            }
        };
    }
    // expand all abstract tasks
    for abstract_id in abstract_tasks.iter() {
        if let Task::Compound(
            CompoundTask { name, methods }
        ) = &*tn.get_task(*abstract_id).borrow() {
            for method in methods.iter() {
                let new_tn = Rc::new(tn.decompose(*abstract_id, method));
                expansions.push(NodeExpansion {
                    connection_label: ConnectionLabel::Decomposition(
                        name.clone(),
                        method.name.clone(),
                    ),
                    tn: new_tn,
                    states: vec![state.clone()]
                });
            }
        }
    }
    expansions
}


#[derive(Debug)]
pub struct NodeExpansion {
    pub connection_label: ConnectionLabel,
    pub tn: Rc<HTN>,
    pub states: Vec<Rc<HashSet<u32>>>,
}

#[derive(Debug)]
pub enum ConnectionLabel {
    Execution(String, u32),
    // task name - method name
    Decomposition(String, String),
}

impl ConnectionLabel {
    pub fn is_decomposition(&self) -> bool {
        match &self {
            ConnectionLabel::Decomposition(_, _) => true,
            _ => false,
        }
    }

    pub fn get_label(&self) -> String {
        match self {
            Self::Execution(name, _) => name.clone(),
            Self::Decomposition(name, method) => name.clone() + &format!("_{}", method),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain_description::DomainTasks, task_network::Method};
    use std::collections::HashMap;

    #[test]
    pub fn expansion_correctness_test() {
        let p1 = Task::Primitive(PrimitiveAction::new(
            "p1".to_string(),
            1,
            HashSet::from([0]),
            vec![HashSet::from([1]), HashSet::from([2, 4])],
            vec![HashSet::from([3]), HashSet::new()],
        ));
        let p2 = Task::Primitive(PrimitiveAction::new(
            "p2".to_string(),
            1,
            HashSet::from([1, 2, 4]),
            vec![HashSet::from([1])],
            vec![HashSet::from([3])],
        ));
        let p3 = Task::Primitive(PrimitiveAction::new(
            "p3".to_string(),
            1,
            HashSet::from([0, 3]),
            vec![HashSet::from([1])],
            vec![HashSet::from([3])],
        ));
        let p4 = Task::Primitive(PrimitiveAction::new(
            "p4".to_string(),
            1,
            HashSet::from([4]),
            vec![HashSet::from([2])],
            vec![HashSet::new()],
        ));
        let t1 = Task::Compound(CompoundTask::new("t1".to_string(), vec![]));
        let domain = Rc::new(DomainTasks::new(vec![p1, p2, p3, p4, t1]));
        let m1 = Method::new(
            "m1".to_string(),
            HTN::new(
                BTreeSet::from([1, 2]),
                vec![(1, 2)],
                domain.clone(),
                HashMap::from([(1, domain.get_id("p1")), (2, domain.get_id("p2"))]),
            ),
        );
        let m2 = Method::new(
            "m2".to_string(),
            HTN::new(
                BTreeSet::from([1, 2]),
                vec![],
                domain.clone(),
                HashMap::from([(1, domain.get_id("p3")), (2, domain.get_id("p4"))]),
            ),
        );
        let id = domain.get_id("t1");
        let domain = domain.add_methods(vec![(id, m1), (id, m2)]);
        let tn = HTN::new(
            BTreeSet::from([1, 2, 3, 4]),
            vec![(1, 4), (2, 4), (3, 4)],
            domain.clone(),
            HashMap::from([
                (1, domain.get_id("p1")),
                (2, domain.get_id("t1")),
                (3, domain.get_id("p3")),
                (4, domain.get_id("p4")),
            ]),
        );
        let state = HashSet::from([0, 3]);
        let expansion = progress(Rc::new(tn), Rc::new(state));
        assert_eq!(expansion.len(), 4);
        let exp_p1: Vec<&NodeExpansion> = expansion
            .iter()
            .filter(|x| match &x.connection_label {
                ConnectionLabel::Execution(x, 1) if x == "p1" => true,
                _ => false,
            })
            .collect();
        assert_eq!(exp_p1.len(), 1);
        assert_eq!(exp_p1[0].states.len(), 2);
        assert_eq!(exp_p1[0].tn.count_tasks(), 3);
        let exp_t1: Vec<&NodeExpansion> = expansion
            .iter()
            .filter(|x| match x.connection_label {
                ConnectionLabel::Decomposition(_, _) => true,
                _ => false,
            })
            .collect();
        assert_eq!(exp_t1.len(), 2);
    }
}
