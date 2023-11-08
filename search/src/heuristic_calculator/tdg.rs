use core::fmt;
use std::{rc::Rc, collections::{HashMap, HashSet, LinkedList, BTreeSet}, vec};

use crate::domain_description::DomainTasks;
use std::cell::RefCell;
use super::{Task, CompoundTask, PrimitiveAction, HTN};

#[derive(Debug)]
pub struct TDG{
    domain: Rc<DomainTasks>,
    root: u32,
    task_vertices: HashMap<u32, Option<Vec<String>>>,
    method_vertices: HashMap<String, Vec<u32>>
}

impl TDG  {
    pub fn new(tn: &HTN) -> TDG {
        if tn.count_tasks() != 1 {
            panic!("TN is not in collapsed format")
        }
        let root = *tn.get_unconstrained_tasks()
                            .iter()
                            .next()
                            .unwrap();
        let domain = tn.domain.clone();
        let root = domain.get_id(&tn.get_task(root).borrow().get_name());
        let mut task_vertices = HashMap::new();
        let mut method_vertices = HashMap::new();
        let mut working_set = LinkedList::from([root]);
        while !working_set.is_empty() {
            let task_id = working_set.pop_front().unwrap();
            match &*domain.get_task(task_id).borrow() {
                Task::Compound(compound) => {
                    match task_vertices.get(&task_id) {
                        Some(_) => { },
                        None => {
                            let mut task_connections = vec![];
                            for (i, method) in compound.methods.iter().enumerate() {
                                let name = format!("task{}_m{}", task_id, i);
                                let subtasks = method.decomposition.get_all_tasks();
                                let subtasks: Vec<u32> = subtasks.iter().map(|x| {
                                    domain.get_id(&x.borrow().get_name())
                                }).collect();
                                method_vertices.insert(name.clone(),subtasks.clone());
                                for elem in subtasks.iter() {
                                    working_set.push_back(*elem);
                                    if !task_vertices.contains_key(elem) && domain.get_task(*elem).borrow().is_primitive() {
                                        task_vertices.insert(elem.clone(), None);
                                    }
                                }
                                task_connections.push(name);
                            }
                            task_vertices.insert(task_id, Some(task_connections));
                        }
                    }
                },
                Task::Primitive(action) => {
                    
                }
            }
        }
        TDG{
            domain: domain,
            root: root,
            task_vertices: task_vertices,
            method_vertices: method_vertices
        }
    }

    // Checks whether "task" can be reached from the current network
    pub fn is_reachable(&self, id: u32) -> bool {
        self.task_vertices.contains_key(&id)
    }

    // compute all reachable tasks from "task"
    pub fn task_reachability(&self, task_id: u32) -> BTreeSet<u32> {
        let mut working_set = LinkedList::from([task_id]);
        let mut result = BTreeSet::from([task_id]);
        while !working_set.is_empty() {
            let current = working_set.pop_front().unwrap();
            match self.task_vertices.get(&current) {
                Some(edges) => {
                    match edges {
                        Some(method_names) => {
                            for method in method_names.iter() {
                                let new_tasks = self.method_vertices.get(method).unwrap();
                                for new_task in new_tasks.iter() {
                                    working_set.push_back(*new_task);
                                    result.insert(*new_task);
                                }
                            }
                        },
                        None => {
                            result.insert(current);
                        }
                    }
                },
                None => { }
            }
        }
        result
    }

    // compute all reachable tasks from an HTN
    pub fn all_reachables(&self, task_ids: &Vec<u32>) -> BTreeSet<u32> {
        let mut reachables = BTreeSet::new();
        for task in task_ids.iter() {
            reachables.extend(self.task_reachability(*task));
        }
        reachables
    }
}

impl fmt::Display for TDG  {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "digraph G {{\n");
        writeln!(f, "\tsubgraph clustertask {{\n\tlabel=\"tasks\"");
        for (task, _) in self.task_vertices.iter() {
            let task_name = self.domain.get_task(*task).borrow().get_name();
            write!(f, "\t\t{}[shape=box]\n", task_name.replace("[", "_").replace("]", "").replace(",", "__"));
        }
        writeln!(f, "\t}}");
        for (task, connections) in self.task_vertices.iter() {
            let task_name = self.domain.get_task(*task).borrow().get_name();
            match connections {
                Some(x) => {
                    for c in x.iter() {
                        write!(f, "\t{} -> {}\n",
                        task_name.replace("[", "_").replace("]", "").replace(",", "__"),
                        c.replace("[", "_").replace("]", "").replace(",", "__"));
                    }
                },
                None => {
                    write!(f, "\t{}\n", task_name.replace("[", "_").replace("]", "").replace(",", "__"));
                }
            }
        }

        writeln!(f, "\tsubgraph clustermethod {{\n\tlabel=\"methods\"");
        for (method, _) in self.method_vertices.iter() {
            write!(f, "\t\t{}[shape=box]\n", method.replace("[", "_").replace("]", "").replace(",", "__"));
        }
        writeln!(f, "\t}}");
        for (name, connections) in self.method_vertices.iter() {
            for x in connections.iter() {
                let task_name = self.domain.get_task(*x).borrow().get_name();
                write!(f, "\t{} -> {}\n", name.replace("[", "_").replace("]", "").replace(",", "__"),
                task_name.replace("[", "_").replace("]", "").replace(",", "__"));
            }
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::{task_network::Method, domain_description::{FONDProblem, Facts}};
    #[test]
    pub fn tdg_correctnes_test() {
        let p1 = Task::Primitive(PrimitiveAction::new(
            "p1".to_string(),
            1,
            HashSet::new(),
            vec![HashSet::from([1])], 
            vec![HashSet::new()]
        ));
        let p2 = Task::Primitive(PrimitiveAction::new(
            "p2".to_string(),
            1,
            HashSet::from([2]),
            vec![HashSet::from([3])], 
            vec![HashSet::new()]
        ));
        let p3 = Task::Primitive(PrimitiveAction::new(
            "p3".to_string(),
            1,
            HashSet::from([3]),
            vec![HashSet::from([2])], 
            vec![HashSet::new()]
        ));
        let p4 = Task::Primitive(PrimitiveAction::new(
            "p4".to_string(),
            1,
            HashSet::from([1]),
            vec![HashSet::from([2]), HashSet::from([3])], 
            vec![HashSet::from([1]), HashSet::from([1])]
        ));
        let t4 = Task::Compound(CompoundTask{
            name: "t4".to_string(),
            methods: vec![] 
        });
        let t3 = Task::Compound(CompoundTask{
            name: "t3".to_string(),
            methods: vec![] 
        });
        let t2 = Task::Compound(CompoundTask{
            name: "t2".to_string(),
            methods: vec![] 
        });
        let t1 = Task::Compound(CompoundTask{
            name: "t1".to_string(),
            methods: vec![] 
        });
        let unreachable_p = Task::Primitive(PrimitiveAction::new(
            "unreach".to_string(),
            1,
            HashSet::from([1]),
            vec![HashSet::from([2]), HashSet::from([3])], 
            vec![HashSet::from([1]), HashSet::from([1])]
        ));
        let unreachable_t = Task::Compound(CompoundTask{
            name: "unreach_t".to_string(),
            methods: vec![] 
        });
        let domain = Rc::new(DomainTasks::new(vec![p1,p2,p3,p4,t1,t2,t3,t4,unreachable_p,unreachable_t]));
        let t4_m = Method::new(
            "t4_m".to_string(),
            HTN::new(
                BTreeSet::from([2, 3]),
                vec![],
                domain.clone(),
                HashMap::from([(2, domain.get_id("p2")), (3, domain.get_id("p3"))
            ]))
        );
        let t3_m = Method::new(
            "t3_m".to_string(),
            HTN::new(
                BTreeSet::from([1, 2]),
                vec![(1,2)],
                domain.clone(),
                HashMap::from([(1, domain.get_id("p2")), (2, domain.get_id("p2"))])
            )
        );
        let t2_m = Method::new(
            "t2_m".to_string(),
            HTN::new(
                BTreeSet::from([4, 3]),
                vec![(4,3)],
                domain.clone(),
                HashMap::from([(4, domain.get_id("p4")), (3, domain.get_id("p3"))])
            )
        );
        let t1_m = Method::new(
            "t1_m".to_string(),
            HTN::new(
                BTreeSet::from([1, 4]),
                vec![],
                domain.clone(),
                HashMap::from([(1, domain.get_id("p1")), (4, domain.get_id("t4"))])
            )
        );
        let unreach_m = Method::new(
            "m_unreach".to_string(),
            HTN::new(
                BTreeSet::from([2]), 
                vec![], 
                domain.clone(),
                HashMap::from([(2, domain.get_id("unreach")),
            ]))
        );
        let domain = domain.add_methods(vec![
            (domain.get_id("t1"), t1_m), (domain.get_id("t2"), t2_m), (domain.get_id("t3"), t3_m),
            (domain.get_id("t4"), t4_m), (domain.get_id("unreach_t"), unreach_m)
        ]);
        let init_tn = HTN::new(
            BTreeSet::from([1,2,3]),
            vec![(1, 3), (2, 3)],
            domain.clone(),
            HashMap::from([
                (1, domain.get_id("t1")), (2, domain.get_id("t2")), (3, domain.get_id("t3"))
            ])
        );
        let mut problem = FONDProblem {
            facts: Facts::new(vec!["1".to_string(), "2".to_string(), "3".to_string()]),
            tasks: domain.clone(),
            initial_state: HashSet::from([]),
            init_tn: init_tn
        };
        problem.collapse_tn();

        let tdg = TDG::new(&problem.init_tn);
        assert_eq!(tdg.is_reachable(domain.get_id("p1")), true);
        assert_eq!(tdg.is_reachable(domain.get_id("t4")), true);
        assert_eq!(tdg.is_reachable(domain.get_id("unreach")), false);
        assert_eq!(tdg.is_reachable(domain.get_id("unreach_t")), false);
        assert_eq!(tdg.task_reachability(domain.get_id("p1")).len(), 1);
        assert_eq!(tdg.task_reachability(domain.get_id("p2")).len(), 1);
        assert_eq!(tdg.task_reachability(domain.get_id("p3")).len(), 1);
        assert_eq!(tdg.task_reachability(domain.get_id("p4")).len(), 1);
        assert_eq!(tdg.task_reachability(domain.get_id("t2")).len(), 3);
        let new_tn = HTN::new(
            BTreeSet::from([1]),
            vec![],
            domain.clone(),
            HashMap::from([(1, domain.get_id("t1"))])
        );
        let mut problem2 = FONDProblem {
            facts: Facts::new(vec!["1".to_string(), "2".to_string(), "3".to_string()]),
            tasks: domain.clone(),
            initial_state: HashSet::from([]),
            init_tn: new_tn
        };
        problem2.collapse_tn();

        let tdg = TDG::new(&problem2.init_tn);
        assert_eq!(tdg.is_reachable(domain.get_id("p3")), true);
        assert_eq!(tdg.is_reachable(domain.get_id("p2")), true);
        assert_eq!(tdg.is_reachable(domain.get_id("p4")), false);
        assert_eq!(tdg.is_reachable(domain.get_id("t1")), true);
        assert_eq!(tdg.is_reachable(domain.get_id("t4")), true);
        let reachables_t4 = tdg.task_reachability(domain.get_id("t4"));
        assert_eq!(reachables_t4.len(), 3);
        assert_eq!(reachables_t4.contains(&domain.get_id("p3")), true);
        assert_eq!(reachables_t4.contains(&domain.get_id("p2")), true);
        assert_eq!(reachables_t4.contains(&domain.get_id("t4")), true);
    }

    #[test]
    pub fn tn_reachability_test() {
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
        let t4 = Task::Compound(CompoundTask{
            name: "t4".to_string(),
            methods: vec![] 
        });
        let t1 = Task::Compound(CompoundTask{
            name: "t1".to_string(),
            methods: vec![] 
        });
        let domain = Rc::new(DomainTasks::new(vec![p1,p2,p3,t4,t1]));
        let t4_m = Method::new(
            "t4_m".to_string(),
            HTN::new(
                BTreeSet::from([2, 3]), 
                vec![],
                domain.clone(),
                HashMap::from([(2, domain.get_id("p2")), (3, domain.get_id("p3"))])
            )
        );
        let t1_m = Method::new(
            "t1_m".to_string(),
            HTN::new(
                BTreeSet::from([1, 4]),
                vec![],
                domain.clone(),
                HashMap::from([(1, domain.get_id("p1")), (4, domain.get_id("t4"))])
            )
        );
        let domain = domain.add_methods(vec![(4,t1_m), (3, t4_m)]);
        let tn = HTN::new(
            BTreeSet::from([1,2,3]),
            vec![(1,3), (2,3)],
            domain.clone(),
            HashMap::from([(1, domain.get_id("p1")), (2,domain.get_id("p2")), (3,domain.get_id("t1"))])
        );
        let mut problem = FONDProblem {
            facts: Facts::new(vec![format!("1"),format!("2"),format!("3"),format!("4"),format!("5")]),
            tasks: domain.clone(),
            initial_state: HashSet::new(),
            init_tn: tn
        };
        problem.collapse_tn();
        let tn = problem.init_tn;
        let tdg = TDG::new(&tn);
        if let Task::Compound(CompoundTask { name, methods }) = &*tn.get_all_tasks()[0].borrow() {
            let new_tn = tn.decompose(*tn.get_nodes().iter().next().unwrap(), &methods[0]);
            let p1_id = new_tn.get_all_tasks_with_ids().iter().filter(|(x,_)| {
                x.borrow().get_name() == "p1"
            }).collect::<Vec<_>>()[0].1;
            let new_tn = new_tn.apply_action(p1_id);
            let task_ids: Vec<_> = new_tn.get_all_task_mappings();
            let result = tdg.all_reachables(&task_ids);
            let result: Vec<_> = result.iter()
                                        .map(|x| domain.get_task(*x).borrow().get_name())
                                        .collect();
            assert_eq!(result.len(), 5);
            assert_eq!(result.contains(&format!("p2")), true);
            assert_eq!(result.contains(&format!("t1")), true);
            assert_eq!(result.contains(&format!("t4")), true);
            assert_eq!(result.contains(&format!("p1")), true);
            assert_eq!(result.contains(&format!("p3")), true);
        };
    }
}