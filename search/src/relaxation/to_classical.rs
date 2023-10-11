use std::collections::HashSet;
use std::rc::Rc;

use super::TDG;
use crate::domain_description::{ClassicalDomain, Facts};
use crate::task_network::{PrimitiveAction, Task};
use crate::{domain_description::FONDProblem, task_network::HTN};
use regex::Regex;

#[derive(Debug)]
pub struct ToClassical {
    tdg: TDG,
    pub domain: ClassicalDomain,
}

impl ToClassical {
    pub fn new(domain: &FONDProblem) -> ToClassical {
        let mut new_facts = domain.facts.clone();
        // top down encoding
        let tasks = domain.tasks.get_all_tasks();
        let top_down_facts = tasks.iter().map(|x| {x.get_name()}).collect();
        new_facts = new_facts.extend(top_down_facts);
        // bottom-up encoding
        let bottom_up_facts: Vec<String> = domain.tasks.get_all_tasks().iter()
            .filter(|x| x.is_primitive())
            .map(|x| x.get_name() + "_reachable")
            .collect();
        new_facts = new_facts.extend(bottom_up_facts);

        let new_actions = ToClassical::encode(&domain, &new_facts);
        let classic_domain = ClassicalDomain { facts: new_facts, actions: new_actions };
        ToClassical { domain: classic_domain, tdg: TDG::new(&domain.init_tn) }
    }

    fn encode(domain: &FONDProblem, facts: &Facts) -> Vec<PrimitiveAction> {
        let mut result = vec![];
        let tasks = domain.tasks.get_all_tasks();
        for task in tasks.iter() {
            match task.as_ref() {
                Task::Compound(c) => {
                    for method in c.methods.iter() {
                        let subtasks = method.decomposition.get_all_tasks();
                        let mut ids = HashSet::new();
                        for subtask in subtasks.iter() {
                            let task_name = subtask.get_name();
                            ids.insert(facts.get_id(&task_name));
                        }
                        let task_id = facts.get_id(&task.get_name());
                        let new_action = PrimitiveAction::new(
                            method.name.clone(),
                            0,
                            ids,
                            vec![HashSet::from([task_id])],
                            vec![HashSet::new()]
                        );
                        result.push(new_action);
                    }
                }
                Task::Primitive(p) => {
                    if p.add_effects.len() > 1 {
                        panic!("Relaxation assumes an all outcome determinized FOND problem");
                    }
                    // action executed effect
                    let mut add_effects = HashSet::from([facts.get_id(&p.name)]);
                    // canonical effects
                    if p.add_effects.len() == 1 {
                        add_effects.extend(p.add_effects[0].clone());
                    }
                    if p.name.contains("__determinized_") {
                        let re = Regex::new(r"__determinized_[0-9]+").unwrap();
                        let cleansed_name = re.replace(&p.name, "__determinized").to_string();
                        let fact_id = facts.get_id(&cleansed_name);
                        add_effects.insert(fact_id);
                    }
                    let top_down_precond = facts.get_id(&(p.name.clone() + "_reachable"));
                    let mut preconds = HashSet::from([top_down_precond]);
                    preconds.extend(p.pre_cond.clone());
                    let new_action = PrimitiveAction::new(
                        p.name.clone(),
                        p.cost,
                        preconds,
                        vec![add_effects],
                        p.del_effects.clone()
                    );
                    result.push(new_action);
                }
            }
        }
        result
    }

    // TODO: Test
    pub fn compute_relaxed_state(&self, tn: &HTN, state: &HashSet<u32>) -> HashSet<u32> {
        let reachables = self.tdg.reachable_from_tn(tn);
        let mut satisfied_preconds = HashSet::new();
        for task in reachables.iter() {
            if let Task::Primitive(prim) = task.as_ref() {
                let mut fact_name = task.get_name();
                if !prim.is_deterministic() {
                    fact_name += "__determinized";
                    let n_effects = prim.add_effects.len() as u32;
                    for i in 0..n_effects {
                        let outcome = fact_name.clone() + "_" + &i.to_string() + "_reachable";
                        let fact_id = self.domain.facts.get_id(&outcome);
                        satisfied_preconds.insert(fact_id);
                    }
                } else {
                    fact_name += "_reachable";
                    let fact_id = self.domain.facts.get_id(&fact_name);
                    satisfied_preconds.insert(fact_id);
                }
            }
        }
        satisfied_preconds.extend(state);
        satisfied_preconds     
    }

    // TODO: test
    pub fn compute_goal_state(&self, tn: &HTN) -> HashSet<u32> {
        let mut goal = HashSet::new();
        for task in tn.get_all_tasks().iter() {
            let mut name = task.get_name();
            if let Task::Primitive(p) = task.as_ref() {
                if !p.is_deterministic() {
                    name += "__determinized";
                }
            }
            let g = self.domain.facts.get_id(&name);
            goal.insert(g);
        }
        goal
    } 
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{Method, CompoundTask};
    use std::collections::HashMap;
    use crate::domain_description::DomainTasks;
    fn generate_problem() -> FONDProblem {
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
            vec![HashSet::from([2])], 
            vec![HashSet::from([1])]
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
        FONDProblem {
            facts: Facts::new(vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()]),
            tasks: DomainTasks::from_rc_tasks(vec![p1, p2, p3, p4, t1.clone(), t2, t3, t4]),
            initial_state: HashSet::new(),
            init_tn: init_tn.clone()
        }
    }

    #[test]
    pub fn encoding_test() {
        let problem = generate_problem();
        let to_classical = ToClassical::new(&problem);
        let encoded = to_classical.domain;
        assert_eq!(encoded.facts.count(), 16);
        assert_eq!(encoded.actions.len(), 8);
        for action in encoded.actions.iter() {
            let mut name = action.name.clone();
            let flag = name.ends_with("_m");
            if flag {
                name = name.replace("_m", "");
            }
            let effect_id = encoded.facts.get_id(&name);
            assert_eq!(action.add_effects[0].contains(&effect_id), true);
            if !flag {
                let precond_id = encoded.facts.get_id(&(name + "_reachable"));
                assert_eq!(action.pre_cond.contains(&precond_id), true);
            }
        }
    }

    #[test]
    pub fn state_computation_test() {
        let problem = generate_problem();
        let to_classical = ToClassical::new(&problem);
        let t1 = &problem.tasks.get_all_tasks().iter()
            .filter(|x| x.get_name() == "t1").cloned().collect::<Vec<Rc<Task>>>()[0];
        let state = HashSet::from([to_classical.domain.facts.get_id("1")]);
        let tn = HTN::new(
            HashSet::from([1]),
            vec![],
            HashMap::from([(1, t1.clone())])
        );
        let relaxed_state = to_classical.compute_relaxed_state(&tn, &state);
        assert_eq!(relaxed_state.len(), 4);
        let names = vec!["P1_reachable", "P2_reachable", "P3_reachable", "1"];
        for fact in relaxed_state {
            let name = to_classical.domain.facts.get_fact(fact);
            let mut is_contained = false;
            for item in names.iter() {
                if name == item {
                    is_contained = true;
                }
            }
            assert_eq!(is_contained, true);
        }

    }
}
