use super::{HTN, Task, PrimitiveAction, CompoundTask};
use crate::{domain_description::{FONDProblem, DomainTasks}, task_network::Method};
use std::{rc::Rc, collections::{HashSet, HashMap}, ops::Index};

pub struct OutcomeDeterminizer {}

impl OutcomeDeterminizer {
    pub fn htn(problem: &FONDProblem) -> FONDProblem {
        // We assume a collapsed network (i.e., with only one init abstract task)
        if problem.init_tn.count_tasks() > 1 {
            panic!("tn not in collapsed format")
        }
        let all_tasks = problem.tasks.get_all_tasks();
        // Identifying non-determinsitic actions in the domain
        let mut nd_actions = HashMap::new();
        for task in all_tasks.iter() {
            if task.is_primitive() {
                if let Task::Primitive(action) = task.as_ref() {
                    if !action.is_deterministic() {
                        let (c, det_actions) = OutcomeDeterminizer::to_abstract(action);
                        let new_task = Task::Compound(c);
                        let det_actions: Vec<Rc<Task>> = det_actions.into_iter()
                            .map(|x| Rc::new(Task::Primitive(x)))
                            .collect();
                        nd_actions.insert(
                            action.name.clone(),
                            (Rc::new(new_task), det_actions)
                        );
                    }
                }
            }
        }
        // Substituing ND-Actions in the domain with their all outcome relaxed version.
        let mut new_tasks = vec![];
        for task in all_tasks.iter() {
            match task.as_ref() {
                Task::Compound(comp) => {
                    let mut new_methods = vec![];
                    for method in comp.methods.clone().iter_mut() {
                        let subtask_names: Vec<(String, u32)> = method.decomposition.get_all_tasks_with_ids()
                            .iter()
                            .map(|(x, y)| {
                                (x.get_name(), *y)
                            }).collect();
                        for (subtask, id) in subtask_names.iter(){
                            if nd_actions.contains_key(subtask) {
                                let (det_comp, _) = nd_actions.get(subtask).unwrap();
                                method.decomposition.change_task(*id, det_comp.clone());
                            }
                        }
                        new_methods.push(method.to_owned());
                    }
                    new_tasks.push(Rc::new(Task::Compound(CompoundTask {
                        name: comp.name.clone(), methods: new_methods })));
                },
                Task::Primitive(prim) => {
                    if prim.is_deterministic() {
                        new_tasks.push(task.clone());
                    } else {
                        let (det_comp, det_prims) = nd_actions.get(&prim.name).unwrap();
                        new_tasks.push(det_comp.clone());
                        new_tasks.extend(det_prims.clone());
                    }
                }
            }
        }
        // Modify the initial task (which is not included in the domain)
        let init_task = &problem.init_tn.get_all_tasks()[0];
        if let Task::Compound(x) = init_task.as_ref() {
            let init_decomposition = &x.methods[0].decomposition;
            let mut new_mappings = HashMap::new();
            for node in init_decomposition.get_nodes().iter() {
                match init_decomposition.get_task(*node).unwrap().as_ref() {
                    Task::Compound(comp) => {
                        new_mappings.insert(*node, init_decomposition.get_task(*node).unwrap());
                    },
                    Task::Primitive(prim) => {
                        if prim.is_deterministic() {
                            new_mappings.insert(*node, init_decomposition.get_task(*node).unwrap());
                        } else {
                            let substitution: Rc<Task> = new_tasks.iter().filter(|t| {
                                let name = init_decomposition.get_task(*node).unwrap().get_name() + "__determinized";
                                t.get_name() == name
                            }).collect::<Vec<&Rc<Task>>>()[0]
                            .clone();
                            new_mappings.insert(*node, substitution);
                        }
                    }
                }
            }
            let new_init_decomposition = init_decomposition.change_mappings(new_mappings);
            let tasks = new_init_decomposition.get_all_tasks();
            let new_top = Rc::new(Task::Compound(CompoundTask {
                name: "__P_G_T_".to_string(),
                methods: vec![
                    Method::new("__P_G_M".to_string(), new_init_decomposition)
                ]
            }));
            let new_tn = HTN::new(
                HashSet::from([1]),
                vec![],
                HashMap::from([(1, new_top)])
            );
            return FONDProblem {
                facts: problem.facts.clone(),
                tasks: DomainTasks::from_rc_tasks(new_tasks),
                initial_state: problem.initial_state.clone(),
                init_tn: new_tn
            }
        };
        unreachable!()
    }

    // Converts a primitive task to an abstract one with several
    // methods for each outcome
    fn to_abstract(action: &PrimitiveAction) -> (CompoundTask, Vec<PrimitiveAction>) {
        let determinized_actions = action.determinize();
        let mut methods = vec![];
        for (i, new_action) in determinized_actions.clone().into_iter().enumerate() {
            let new_method = Method::new(
                new_action.name.clone() + "__method_" + &i.to_string(),
                HTN::new(
                    HashSet::from([1]),
                    vec![],
                    HashMap::from(
                        [(1, Rc::new(Task::Primitive(new_action)))]
                    )
                )
            );
            methods.push(new_method);
        }
        let c = CompoundTask { name: action.name.clone() + "__determinized", methods: methods };
        (c, determinized_actions)
    }

    fn method_subsititution() {
        
    }
}

#[cfg(test)]
mod tests {
    use crate::domain_description::Facts;
    use super::*;

    #[test]
    pub fn determinization_test() {
        let p1 = Rc::new(Task::Primitive(PrimitiveAction::new(
            "p1".to_string(),
            1,
            HashSet::from([1]),
            vec![HashSet::from([2])], 
            vec![HashSet::from([1])]
        )));
        let p2 = Rc::new(Task::Primitive(PrimitiveAction::new(
            "p2".to_string(),
            1,
            HashSet::from([1]),
            vec![HashSet::from([2]), HashSet::from([2, 5])], 
            vec![HashSet::from([3]), HashSet::from([4])]
        )));
        let p3 = Rc::new(Task::Primitive(PrimitiveAction::new(
            "p3".to_string(),
            1,
            HashSet::from([2]),
            vec![HashSet::new(),], 
            vec![HashSet::new(),]
        )));
        let t1 = Rc::new(Task::Compound(CompoundTask{
            name: "t1".to_string(),
            methods: vec![
                Method::new(
                    "t1_m".to_string(),
                    HTN::new(HashSet::from([2, 3]), vec![], HashMap::from([
                        (2, Rc::clone(&p2)), (3, Rc::clone(&p3))
                    ]))
                )
            ] 
        }));
        let tn = HTN::new(
            HashSet::from([1,2,3]),
            vec![(1,3), (2,3)],
            HashMap::from(
                [(1, Rc::clone(&p1)), (2, Rc::clone(&t1)), (3, Rc::clone(&p2))]
            )
        ).collapse_tn();
        let state = HashSet::from([1]);
        let domain_tasks = DomainTasks::from_rc_tasks(vec![p1, p2, p3, t1]);
        let facts = Facts::new(vec!["pre1".to_string(), "pre2".to_string()]);
        let problem = FONDProblem {
            facts: facts,
            tasks: domain_tasks,
            initial_state: state,
            init_tn: tn
        };
        let relaxed = OutcomeDeterminizer::htn(&problem);
        assert_eq!(relaxed.facts.count(), problem.facts.count());
        let new_tasks = relaxed.tasks.get_all_tasks();
        assert_eq!(new_tasks.len(), 6);
        for task in new_tasks.iter() {
            if task.get_name() == "p2__determinized" {
                if let Task::Compound(CompoundTask { name, methods }) = task.as_ref() {
                    assert_eq!(methods.len(), 2);
                } else {
                    panic!()
                }
            }
        }
        let top = relaxed.init_tn.get_task(1).unwrap();
        assert_eq!(top.get_name(), "__P_G_T_");
        if let Task::Compound(t) = top.as_ref() {
            assert_eq!(t.methods.len(), 1);
            let decomp = &t.methods[0].decomposition;
            assert_eq!(decomp.count_tasks(), 3);
            let task_names: Vec<String> = decomp.get_all_tasks().iter().map(|x| x.get_name()).collect();
            assert_eq!(task_names.contains(&"p1".to_string()), true);
            assert_eq!(task_names.contains(&"p2__determinized".to_string()), true);
            assert_eq!(task_names.contains(&"t1".to_string()), true)
        }
        assert_eq!(relaxed.init_tn.count_tasks(), problem.init_tn.count_tasks());
    }
}