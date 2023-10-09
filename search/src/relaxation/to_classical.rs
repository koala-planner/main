use std::collections::HashSet;
use std::rc::Rc;

use super::TDG;
use crate::domain_description::{ClassicalDomain, Facts};
use crate::task_network::{PrimitiveAction, Task};
use crate::{domain_description::FONDProblem, task_network::HTN};

#[derive(Debug)]
pub struct ToClassical {
    tdg: TDG,
    domain: FONDProblem,
}

impl ToClassical {
    pub fn new(domain: FONDProblem) -> ToClassical {
        let tdg = TDG::new(&domain.init_tn);
        ToClassical { domain, tdg: tdg }
    }

    // TODO: test
    pub fn encode(&self, tn: &HTN, state: &HashSet<u32>) -> ClassicalDomain {
        let all_reachables = self.tdg.reachable_from_tn(tn);
        let reachable_primitives = all_reachables
            .iter()
            .filter(|x| x.is_primitive())
            .cloned()
            .collect();
        let reachable_compounds: HashSet<Rc<Task>> = all_reachables
            .difference(&reachable_primitives)
            .cloned()
            .collect();
        // augmentation of literals
        let new_facts: Vec<String> = all_reachables
            .iter()
            .map(|x| "reached_".to_owned() + &x.get_name())
            .collect();
        let new_facts = self.domain.facts.extend(new_facts);
        // encode methods
        let mut new_actions = ToClassical::encode_methods(
            &new_facts, reachable_compounds);
        // augment primitive action effects
        new_actions.extend(ToClassical::augment_primitive_effects(&new_facts, reachable_primitives));
        // construct goal state
        let active_tasks = tn.get_all_tasks();
        let goal_state: HashSet<u32> = active_tasks
            .iter()
            .map(|x| new_facts.get_id(&("reached_".to_owned() + &x.get_name())))
            .collect();
        ClassicalDomain::new(new_facts, new_actions, state.clone(), goal_state)
    }

    // encode all methods as classical actions
    // each method is translated into one primitive action
    fn encode_methods(facts: &Facts,
        reachable_compounds: HashSet<Rc<Task>>) -> Vec<PrimitiveAction> {
        let mut new_actions: Vec<PrimitiveAction> = vec![];
        for compound in reachable_compounds.iter() {
            if let Task::Compound(compound) = compound.as_ref() {
                for method in compound.methods.iter() {
                    let subtasks = method.decomposition.get_all_tasks();
                    let precond: HashSet<u32> = subtasks
                        .iter()
                        .map(|x| {
                            let literal = "reached_".to_owned() + &x.get_name();
                            facts.get_id(&literal)
                        })
                        .collect();
                    let effect = facts.get_id(&("reached_".to_owned() + &compound.name));
                    let new_action = PrimitiveAction::new(
                        compound.name.clone() + "_" + &method.name,
                        0,
                        precond,
                        vec![HashSet::from([effect])],
                        vec![HashSet::new()],
                    );
                    new_actions.push(new_action);
                }
            }
        }
        new_actions
    }

    fn augment_primitive_effects(facts: &Facts, reachable_primitives: HashSet<Rc<Task>>) -> Vec<PrimitiveAction> {
        let mut new_actions = vec![];
        // augment primitive action effects
        for primitive in reachable_primitives.into_iter() {
            if let Task::Primitive(primitive) = primitive.as_ref() {
                let new_effect = facts.get_id(&("reached_".to_owned() + &primitive.name));
                let new_action =
                    primitive.augment(HashSet::from([new_effect]), HashSet::new(), HashSet::new());
                new_actions.push(new_action);
            }
        }
        new_actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{Method, CompoundTask};
    use std::collections::HashMap;
    use crate::domain_description::DomainTasks;
    use crate::heuristic_calculator::AStar;
    #[test]
    // TODO: Fix
    pub fn encoding_test() {
        let p1 = Rc::new(Task::Primitive(PrimitiveAction::new(
            "P1".to_string(),
            1,
            HashSet::new(),
            vec![HashSet::from([1])], 
            vec![HashSet::new()]
        )));
        let p2 = Rc::new(Task::Primitive(PrimitiveAction::new(
            "P2".to_string(),
            1,
            HashSet::from([2]),
            vec![HashSet::from([3])], 
            vec![HashSet::new()]
        )));
        let p3 = Rc::new(Task::Primitive(PrimitiveAction::new(
            "P3".to_string(),
            1,
            HashSet::from([3]),
            vec![HashSet::from([2])], 
            vec![HashSet::new()]
        )));
        let p4 = Rc::new(Task::Primitive(PrimitiveAction::new(
            "P4".to_string(),
            1,
            HashSet::from([1]),
            vec![HashSet::from([2]), HashSet::from([3])], 
            vec![HashSet::from([1]), HashSet::from([1])]
        )));
        let t4 = Rc::new(Task::Compound(CompoundTask{
            name: "t4".to_string(),
            methods: vec![
                Method::new(
                    "t4_m".to_string(),
                    HTN::new(HashSet::from([2, 3]), vec![], HashMap::from([
                        (2, Rc::clone(&p2)), (3, Rc::clone(&p3))
                    ]))
                )
            ] 
        }));
        let t3 = Rc::new(Task::Compound(CompoundTask{
            name: "t3".to_string(),
            methods: vec![
                Method::new(
                    "t3_m".to_string(),
                    HTN::new(
                        HashSet::from([1, 2]),
                        vec![(1,2)],
                        HashMap::from([
                            (1, Rc::clone(&p2)), (2, Rc::clone(&p2))
                        ])
                    )
                )
            ] 
        }));
        let t2 = Rc::new(Task::Compound(CompoundTask{
            name: "t2".to_string(),
            methods: vec![
                Method::new(
                    "t2_m".to_string(),
                    HTN::new(
                        HashSet::from([4, 3]),
                        vec![(4,3)],
                        HashMap::from([
                            (4, Rc::clone(&p4)), (3, Rc::clone(&p3))
                        ])
                    )
                )
            ] 
        }));
        let t1 = Rc::new(Task::Compound(CompoundTask{
            name: "t1".to_string(),
            methods: vec![
                Method::new(
                    "t1_m".to_string(),
                    HTN::new(
                        HashSet::from([1, 4]),
                        vec![],
                        HashMap::from([
                            (1, Rc::clone(&p1)), (4, Rc::clone(&t4))
                        ])
                    )
                )
            ] 
        }));
        let init_tn = HTN::new(
            HashSet::from([1,2,3]),
            vec![(1, 3), (2, 3)],
            HashMap::from([
                (1, Rc::clone(&t1)), (2, Rc::clone(&t2)), (3, Rc::clone(&t3))
            ])
        ).collapse_tn();
        let problem = FONDProblem {
            facts: Facts::new(vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()]),
            tasks: DomainTasks::from_rc_tasks(vec![p1, p2, p3, p4, t1.clone(), t2, t3, t4]),
            initial_state: HashSet::new(),
            init_tn: init_tn.clone()
        };
        let to_classical = ToClassical::new(problem);
        let encoded = to_classical.encode(&init_tn, &HashSet::new());
        assert_eq!(encoded.facts.count(), 13);
        assert_eq!(encoded.facts.contains(&"reached_P1".to_owned()), true);
        assert_eq!(encoded.facts.contains(&"reached_P2".to_owned()), true);
        assert_eq!(encoded.facts.contains(&"reached_P3".to_owned()), true);
        assert_eq!(encoded.facts.contains(&"reached_P4".to_owned()), true);
        assert_eq!(encoded.facts.contains(&"reached_t1".to_owned()), true);
        assert_eq!(encoded.facts.contains(&"reached_t2".to_owned()), true);
        assert_eq!(encoded.facts.contains(&"reached_t3".to_owned()), true);
        assert_eq!(encoded.facts.contains(&"reached_t4".to_owned()), true);
        assert_eq!(encoded.actions.len(), 9);
        let tn_2 = HTN::new(
            HashSet::from([1]),
            vec![],
            HashMap::from([(1, Rc::clone(&t1))])
        );
        let encoded_2 = to_classical.encode(&tn_2, &HashSet::new());
        assert_eq!(encoded_2.facts.count(), 9);
        assert_eq!(encoded_2.actions.len(), 5);
    }
}
