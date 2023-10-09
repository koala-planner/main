use std::{rc::Rc, collections::{HashMap, HashSet, LinkedList}, vec};

use super::{Task, CompoundTask, PrimitiveAction, HTN};

#[derive(Debug)]
pub struct TDG {
    root: Rc<Task>,
    task_vertices: HashMap<Rc<Task>, Option<Vec<String>>>,
    method_vertices: HashMap<String, Vec<Rc<Task>>>
}

impl TDG {
    pub fn new(tn: &HTN) -> TDG {
        let tn = tn.collapse_tn();
        let root = *tn.get_unconstrained_tasks()
                            .iter()
                            .next()
                            .unwrap();
        let root = tn.get_task(root).unwrap();
        let mut task_vertices = HashMap::new();
        let mut method_vertices = HashMap::new();
        let mut working_set = LinkedList::from([Rc::clone(&root)]);
        while !working_set.is_empty() {
            let task = working_set.pop_front().unwrap();
            match task.as_ref() {
                Task::Compound(compound) => {
                    match task_vertices.get(&task) {
                        Some(_) => { },
                        None => {
                            let mut task_connections = vec![];
                            for method in compound.methods.iter() {
                                let name = task.get_name() + " " + &method.name;
                                let subtasks = method.decomposition.get_all_tasks();
                                method_vertices.insert(name.clone(),subtasks.clone());
                                for elem in subtasks.iter() {
                                    working_set.push_back(Rc::clone(elem));
                                    if !task_vertices.contains_key(elem) && elem.as_ref().is_primitive() {
                                        task_vertices.insert(elem.clone(), None);
                                    }
                                }
                                task_connections.push(name);
                            }
                            task_vertices.insert(
                                task,
                                Some(task_connections)
                            );
                        }
                    }
                },
                Task::Primitive(action) => {
                    
                }
            }
        }
        TDG {
            root: root,
            task_vertices: task_vertices,
            method_vertices: method_vertices
        }
    }

    // Checks whether "task" can be reached from the current network
    pub fn is_reachable(&self, task: &Task) -> bool {
        self.task_vertices.contains_key(task)
    }

    // compute all reachable tasks from "task"
    pub fn all_reachables(&self, task: Rc<Task>) -> HashSet<Rc<Task>> {
        let mut working_set = LinkedList::from([task.clone()]);
        let mut result = HashSet::from([task]);
        while !working_set.is_empty() {
            let current = working_set.pop_front().unwrap();
            match self.task_vertices.get(&current) {
                Some(edges) => {
                    match edges {
                        Some(method_names) => {
                            for method in method_names.iter() {
                                let new_tasks = self.method_vertices.get(method).unwrap();
                                for new_task in new_tasks.iter() {
                                    working_set.push_back(Rc::clone(new_task));
                                    result.insert(Rc::clone(new_task));
                                }
                            }
                        },
                        None => {
                            result.insert(Rc::clone(&current));
                        }
                    }
                },
                None => { }
            }
        }
        result
    }

    // compute all reachable tasks from an HTN
    pub fn reachable_from_tn(&self, tn: &HTN) -> HashSet<Rc<Task>> {
        let all_tasks = tn.get_all_tasks();
        let mut reachables = HashSet::new();
        for task in all_tasks.iter() {
            reachables.extend(self.all_reachables(task.clone()));
        }
        reachables
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_network::Method;
    #[test]
    pub fn tdg_correctnes_test() {
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
        let tdg = TDG::new(&init_tn);
        let unreachable_p = Rc::new(Task::Primitive(PrimitiveAction::new(
            "unreach".to_string(),
            1,
            HashSet::from([1]),
            vec![HashSet::from([2]), HashSet::from([3])], 
            vec![HashSet::from([1]), HashSet::from([1])]
        )));
        let unreachable_t = Rc::new(Task::Compound(CompoundTask{
            name: "unreach_t".to_string(),
            methods: vec![
                Method::new(
                    "m_unreach".to_string(),
                    HTN::new(HashSet::from([2]), vec![], HashMap::from([
                        (2, Rc::clone(&unreachable_p)),
                    ]))
                )
            ] 
        }));
        assert_eq!(tdg.is_reachable(p1.as_ref()), true);
        assert_eq!(tdg.is_reachable(t4.as_ref()), true);
        assert_eq!(tdg.is_reachable(unreachable_p.as_ref()), false);
        assert_eq!(tdg.is_reachable(unreachable_t.as_ref()), false);
        assert_eq!(tdg.all_reachables(Rc::clone(&p1)).len(), 1);

        let new_tn = HTN::new(
            HashSet::from([1]),
            vec![],
            HashMap::from([
                (1, Rc::clone(&t1))
            ])
        ).collapse_tn();
        let tdg = TDG::new(&new_tn);
        assert_eq!(tdg.is_reachable(p3.as_ref()), true);
        assert_eq!(tdg.is_reachable(p2.as_ref()), true);
        assert_eq!(tdg.is_reachable(p4.as_ref()), false);
        assert_eq!(tdg.is_reachable(t1.as_ref()), true);
        assert_eq!(tdg.is_reachable(t4.as_ref()), true);
        assert_eq!(tdg.all_reachables(Rc::clone(&t4)).len(), 3);
        assert_eq!(tdg.all_reachables(Rc::clone(&t4)).contains(&p3), true);
        assert_eq!(tdg.all_reachables(Rc::clone(&t4)).contains(&p2), true);
        assert_eq!(tdg.all_reachables(Rc::clone(&t4)).contains(&t4), true);
    }

    #[test]
    pub fn tn_reachability_test() {
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
        let tn = HTN::new(
            HashSet::from([1,2,3]),
            vec![(1,3), (2,3)],
            HashMap::from([(1, p1), (2,p2), (3,t1)])
        );
        
        let tdg = TDG::new(&tn);
        let new_tn = tn.apply_action(1);
        let result = tdg.reachable_from_tn(&new_tn);
        assert_eq!(result.len(), 5);
    }
}