use super::{HTN, Task, PrimitiveAction, CompoundTask};
use crate::{domain_description::{FONDProblem, DomainTasks}, task_network::Method};
use std::{rc::{Rc, self}, collections::{HashSet, HashMap, BTreeSet}, ops::Index, cell::RefCell};

pub struct OutcomeDeterminizer {}

impl OutcomeDeterminizer {
    pub fn htn(problem: &FONDProblem) -> FONDProblem {
        // We assume a collapsed network (i.e., with only one init abstract task)
        if problem.init_tn.count_tasks() > 1 {
            panic!("tn not in collapsed format")
        }
        let all_tasks = problem.tasks.get_all_tasks();
        // find non deterministic tasks in the domain
        let nd_actions = OutcomeDeterminizer::determinize_nd_tasks(all_tasks);
        // substitue nd tasks
        let new_tasks = OutcomeDeterminizer::substitue_nd_tasks(all_tasks, &nd_actions);
        let new_domain = DomainTasks::new(new_tasks);
        let nd_act_map: Vec<(u32, Vec<u32>)> = nd_actions.iter().map(|(_, (c, det_acts))| {
            (new_domain.get_id(&c.get_name()),
             det_acts.iter().map(|x| new_domain.get_id(&x.get_name())).collect::<Vec<_>>())
        }).collect();

        let mut rc_domain = Rc::new(new_domain);
        let methods = OutcomeDeterminizer::construct_methods(rc_domain.clone(), &nd_act_map);
        rc_domain = rc_domain.add_methods(methods).clone();

        // Create init tn (we know that the task is in collapsed format)
        let top_task = problem.init_tn.get_all_tasks().iter().next().unwrap().borrow().get_name();
        let new_top_id = rc_domain.get_id(&top_task);
        let new_tn = HTN::new(
            BTreeSet::from([1]),
            vec![],
            rc_domain.clone(),
            HashMap::from([(1, new_top_id)])
        );
        FONDProblem { 
            facts: problem.facts.clone(),
            tasks: rc_domain,
            initial_state: problem.initial_state.clone(),
            init_tn: new_tn
        }
    }

    // Converts a primitive task to an abstract one with several
    // methods for each outcome
    fn to_abstract(action: &PrimitiveAction) -> (CompoundTask, Vec<PrimitiveAction>) {
        let determinized_actions = action.determinize();
        let c = CompoundTask { name: action.name.clone() + "__determinized", methods: vec![] };
        (c, determinized_actions)
    }

    fn construct_methods(
        domain: Rc<DomainTasks>,
        determinized_tasks: &Vec<(u32, Vec<u32>)>
    ) -> Vec<(u32, Method)> {
        let mut methods = vec![];
        for (det_c, det_acts) in determinized_tasks.iter() {
            for act in det_acts.iter() {
                let action = domain.get_task(*act);
                let new_method = Method::new(
                    format!("m_{}", action.borrow().get_name()),
                    HTN::new(
                        BTreeSet::from([1]), 
                        vec![], 
                        domain.clone(),
                        HashMap::from([(1, *act)])
                    )
                );
                methods.push((*det_c, new_method));
            }
        }
        methods
    }

    fn determinize_nd_tasks(all_tasks: &Vec<RefCell<Task>>) -> HashMap<usize, (Task, Vec<Task>)>{
        // Identifying non-determinsitic actions in the domain
        let mut nd_actions = HashMap::new();
        for (task_id, task) in all_tasks.iter().enumerate() {
            match &*task.borrow() {
                Task::Compound(_) => {},
                Task::Primitive(action) => {
                    if !action.is_deterministic() {
                        let (c, det_actions) = OutcomeDeterminizer::to_abstract(&action);
                        let det_actions: Vec<Task> = det_actions.into_iter()
                                                                .map(|x| Task::Primitive(x))
                                                                .collect();
                        nd_actions.insert(task_id,(Task::Compound(c), det_actions));
                    }
                }
            };
        }
        nd_actions
    }

    fn substitue_nd_tasks(all_tasks: &Vec<RefCell<Task>>, nd_actions: &HashMap<usize, (Task, Vec<Task>)>) -> Vec<Task>{
        let mut new_tasks: Vec<Task> = all_tasks.iter()
                                                .enumerate()
                                                .filter(|(i, t)| {
                                                    !nd_actions.contains_key(i)
                                                }).map(|(i, x)| x.borrow().clone())
                                                .collect();
        // add all-outcome determinized versions to the domain
        new_tasks.extend(nd_actions.clone()
                                        .into_iter()
                                        .map(|(_, (c, det_acts))| {
                                            let mut new_tasks = vec![c];
                                            new_tasks.extend(det_acts);
                                            new_tasks
                                        }).flatten()
                                        .collect::<Vec<Task>>());
        let mut bijection =  HashMap::new();
        for (prev_id, prev_task) in all_tasks.iter().enumerate() {
            let borrowed_task = prev_task.borrow();
            let new_name = match &*borrowed_task {
                Task::Compound(_) => {
                    borrowed_task.get_name()
                },
                Task::Primitive(p) => {
                    if !p.is_deterministic() {
                        p.name.clone() + "__determinized"
                    } else {
                        p.name.clone()
                    }
                }
            };
            let new_id = new_tasks.iter().position(|x| x.get_name() == new_name).unwrap();
            bijection.insert(prev_id as u32, new_id as u32);
        }
        // Change the mappings of exisiting methods
        for t in new_tasks.iter_mut() {
            if let Task::Compound(CompoundTask { name, methods }) = t {
                for m in methods.iter_mut() {
                    for (node_id, task_id) in m.decomposition.mappings.iter_mut() {
                        let new_task_id = *bijection.get(task_id).unwrap();
                        *task_id = new_task_id;
                    }
                }
            }
        }
        new_tasks
    }
}

#[cfg(test)]
mod tests {
    use crate::domain_description::Facts;
    use super::*;

    #[test]
    pub fn determinization_test() {
        let p1 = Task::Primitive(PrimitiveAction::new(
            "p1".to_string(),
            1,
            HashSet::from([1]),
            vec![HashSet::from([2])], 
            vec![HashSet::from([1])]
        ));
        let p2 = Task::Primitive(PrimitiveAction::new(
            "p2".to_string(),
            1,
            HashSet::from([1]),
            vec![HashSet::from([2]), HashSet::from([2, 5])], 
            vec![HashSet::from([3]), HashSet::from([4])]
        ));
        let p3 = Task::Primitive(PrimitiveAction::new(
            "p3".to_string(),
            1,
            HashSet::from([2]),
            vec![HashSet::new(),], 
            vec![HashSet::new(),]
        ));
        let t1 = Task::Compound(CompoundTask{
            name: "t1".to_string(),
            methods: vec![] 
        });
        let domain = Rc::new(DomainTasks::new(vec![p1,p2,p3,t1]));
        let t1_m = Method::new(
            "t1_m".to_string(),
            HTN::new(
                BTreeSet::from([2, 3]),
                vec![],
                domain.clone(),
                HashMap::from([(2, domain.get_id("p2")), (3, domain.get_id("p3"))])
            )
        );
        let domain = domain.add_methods(vec![(3, t1_m)]);
        let tn = HTN::new(
            BTreeSet::from([1,2,3]),
            vec![(1,3), (2,3)],
            domain.clone(),
            HashMap::from(
                [(1, domain.get_id("p1")), (2, domain.get_id("t1")), (3, domain.get_id("p2"))]
            )
        );
        let state = HashSet::from([1]);
        let facts = Facts::new(vec!["pre1".to_string(), "pre2".to_string()]);
        let mut problem = FONDProblem {
            facts: facts,
            tasks: domain.clone(),
            initial_state: state,
            init_tn: tn
        };
        problem.collapse_tn();
        let relaxed = OutcomeDeterminizer::htn(&problem);
        assert_eq!(relaxed.facts.count(), problem.facts.count());
        let new_tasks: Vec<String> = relaxed.tasks.get_all_tasks().iter()
                                        .map(|x| x.borrow().get_name())
                                        .collect();
        assert_eq!(new_tasks.len(), 7);
        assert_eq!(new_tasks.contains(&format!("p1")), true);
        assert_eq!(new_tasks.contains(&format!("p2__determinized")), true);
        assert_eq!(new_tasks.contains(&format!("p2__determinized_0")), true);
        assert_eq!(new_tasks.contains(&format!("p2__determinized_1")), true);
        assert_eq!(new_tasks.contains(&format!("p3")), true);
        assert_eq!(new_tasks.contains(&format!("t1")), true);
        let id_p2 = relaxed.tasks.get_id("p2__determinized");
        if let Task::Compound(CompoundTask { name, methods }) = &*relaxed.tasks.get_task(id_p2).borrow() {
            let mut det_acts = vec![relaxed.tasks.get_id("p2__determinized_0"), relaxed.tasks.get_id("p2__determinized_1")];
            assert_eq!(methods.len(), 2);
            for m in methods.iter() {
                assert_eq!(m.decomposition.get_nodes().len(), 1);
                let decomp_node = *m.decomposition.get_nodes().iter().next().unwrap();
                let decomp_node = m.decomposition.get_task(decomp_node).borrow().get_name();
                let decomp_node = relaxed.tasks.get_id(&decomp_node);
                assert_eq!(det_acts.contains(&decomp_node), true);
                det_acts = det_acts.into_iter().filter(|x| *x != decomp_node).collect()
            }
        } else {
            panic!("Task is not compound")
        };
        if let Task::Compound(t) = &*relaxed.init_tn.get_all_tasks()[0].borrow() {
            assert_eq!(t.methods.len(), 1);
            let decomp = &t.methods[0].decomposition;
            assert_eq!(decomp.count_tasks(), 3);
            let task_names: Vec<String> = decomp.get_all_tasks().iter().map(|x| x.borrow().get_name()).collect();
            assert_eq!(task_names.contains(&"p1".to_string()), true);
            assert_eq!(task_names.contains(&"p2__determinized".to_string()), true);
            assert_eq!(task_names.contains(&"t1".to_string()), true)
        };
        assert_eq!(relaxed.init_tn.count_tasks(), problem.init_tn.count_tasks());
    }
}