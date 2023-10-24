use crate::{domain_description::{FONDProblem, DomainTasks}, task_network::{Task, HTN}};
use std::{rc::Rc, collections::{HashMap, BTreeSet}};
use super::{CompoundTask, Method};
use std::collections::HashSet;
pub struct DeleteRelaxation {

}

impl DeleteRelaxation {
    pub fn htn(problem: &FONDProblem) -> FONDProblem {
        let tasks = problem.tasks.get_all_tasks();
        let mut new_tasks = vec![];
        for task in tasks.iter() {
            match task.as_ref() {
                Task::Compound(a) => {
                    new_tasks.push(task.clone());
                },
                Task::Primitive(a) => {
                    let new_action = a.delete_relax();
                    let new_action = Rc::new(Task::Primitive(new_action));
                    new_tasks.push(new_action);
                }
            }
        }
        // We assume a collapsed network (i.e., with only one init abstract task)
        // substitue initial mappings with new mappings
        if problem.init_tn.count_tasks() > 1 {
            panic!("network not in collapsed format")
        }
        let init_task = &problem.init_tn.get_all_tasks()[0];
        if let Task::Compound(t1) = init_task.as_ref() {
            let init_tn = &t1.methods[0].decomposition;
            let mut new_mappings = HashMap::new();
            for subtask_id in init_tn.get_nodes().iter() {
                if init_tn.is_primitive(*subtask_id) {
                    let substitue = new_tasks.iter().filter(|x| {
                        let name = init_tn.get_task(*subtask_id).unwrap().get_name();
                        x.get_name() == (name + "__delete_relaxed")
                    }).collect::<Vec<&Rc<Task>>>()[0];
                    new_mappings.insert(*subtask_id, substitue.clone());
                }
                else {
                    new_mappings.insert(*subtask_id, init_tn.get_task(*subtask_id).unwrap().clone());
                }
            }
            // create new initial tn
            let new_decomposition = problem.init_tn.change_mappings(new_mappings);
            let new_top = Rc::new(Task::Compound(CompoundTask {
                name: "__P_G_T_".to_string(),
                methods: vec![
                    Method::new("__P_G_M".to_string(), new_decomposition)
                ]
            }));
            let new_tn = HTN::new(
                BTreeSet::from([1]),
                vec![],
                HashMap::from([(1, new_top)])
            );
            return FONDProblem {
                facts: problem.facts.clone(),
                tasks: DomainTasks::from_rc_tasks(new_tasks),
                initial_state: problem.initial_state.clone(),
                init_tn: new_tn
            }
        }
        unreachable!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_description::Facts;
    use crate::task_network::{CompoundTask, PrimitiveAction, Task, Applicability};
    #[test]
    pub fn delete_relax_test() {
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
                    HTN::new(BTreeSet::from([2, 3]), vec![], HashMap::from([
                        (2, Rc::clone(&p2)), (3, Rc::clone(&p3))
                    ]))
                )
            ] 
        }));
        let tn = HTN::new(
            BTreeSet::from([1,2,3]),
            vec![(1,3), (2,3)],
            HashMap::from(
                [(1, Rc::clone(&p1)), (2, Rc::clone(&t1)), (3, Rc::clone(&p3))]
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
        let relaxed = DeleteRelaxation::htn(&problem);
        assert_eq!(relaxed.facts.count(), problem.facts.count());
        assert_eq!(relaxed.tasks.get_all_tasks().len(), problem.tasks.get_all_tasks().len());
        let tasks = relaxed.tasks.get_all_tasks();
        assert_eq!(tasks.len(), problem.tasks.get_all_tasks().len());
        for task in tasks.iter() {
            if let Task::Primitive(x) = task.as_ref() {
                assert_eq!(x.is_applicable(&HashSet::from([0,1,2,3,4,5])), true);
            }
        }
    }
}