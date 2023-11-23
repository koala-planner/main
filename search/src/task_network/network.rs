use crate::domain_description::DomainTasks;

use super::Graph;
use super::task_structs::{CompoundTask, Method, PrimitiveAction, Task};
use std::collections::{HashMap, HashSet, BTreeSet};
use std::fmt::{self, write};
use rand::distributions::DistString;
use std::hash::Hash;
use std::rc::{Rc, Weak};
use rand::{distributions::Alphanumeric, Rng};
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct HTN {
    network: Graph,
    pub domain: Rc<DomainTasks>, // TODO: Convert to Weak
    // A mapping from task id in the network to its ID in the domain
    pub mappings: HashMap<u32, u32>,
}

impl HTN {
    pub fn new(
        tasks: BTreeSet<u32>,
        orderings: Vec<(u32, u32)>,
        domain: Rc<DomainTasks>,
        mappings: HashMap<u32, u32>,
    ) -> HTN {
        HTN {
            network: Graph::new(tasks, orderings),
            mappings,
            domain: domain
        }
    }

    pub fn get_orderings(&self) -> Vec<(u32,u32)>{
        self.network.get_edges()
    }

    pub fn get_all_tasks(&self) -> Vec<&RefCell<Task>> {
        self.network.nodes.iter().map(|id| {
            self.get_task(*id)
        }).collect()
    }

    // Counts the number of tasks in the network grouped by their mapping
    pub fn count_tasks_with_frequency(&self) -> HashMap<u32, u32> {
        let task_ids: Vec<_> = self.mappings.iter().map(|(_, v)| {
            *v
        }).collect();
        let mut result = HashMap::new();
        for id in task_ids.iter() {
            if result.contains_key(id) {
                let val = result.get_mut(id).unwrap();
                *val += 1;
            } else {
                result.insert(*id, 1);
            }
        }
        result
    }

    pub fn get_all_tasks_with_ids(&self) -> Vec<(&RefCell<Task>, u32)> {
        self.network.nodes.iter().map(|id| {
            (self.get_task(*id), *id)
        }).collect()
    }

    pub fn count_tasks(&self) -> usize {
        self.network.count_nodes()
    }

    pub fn is_empty(&self) -> bool {
        self.network.count_nodes() == 0
    }

    pub fn get_task(&self, node_id: u32) -> &std::cell::RefCell<Task> {
        match self.mappings.get(&node_id) {
            Some(x) => {self.domain.get_task(*x)},
            None => {panic!("task not in the network")}
        }
    }

    pub fn get_unconstrained_tasks(&self) -> BTreeSet<u32> {
        self.network.get_unconstrained_nodes()
    }

    pub fn get_incoming_edges(&self, id: u32) -> BTreeSet<u32> {
        self.network.get_incoming_edges(id)
    }

    pub fn decompose(&self, id: u32, method: &Method) -> HTN {
        match &*self.get_task(id).borrow() {
            Task::Primitive(_) => {
                panic!("task is primitive");
            },
            _ => {}
        }
        // Changing IDs
        let network_max_id = self.network.nodes.iter().max().unwrap();
        let subgraph_max_id = method.decomposition.network.nodes.iter().max().unwrap();
        let max_id = *network_max_id.max(subgraph_max_id) + 1;
        let relabeled_subgraph = HTN::relabel_nodes(&method.decomposition, max_id);
        // Creating Graph
        let mut new_graph = self.network.clone();
        let outgoing_edges = self.network.get_outgoing_edges(id);
        let incoming_edges = self.network.get_incoming_edges(id);
        new_graph = new_graph.remove_node(id);
        new_graph = new_graph.add_subgraph(
            Graph::new(
                relabeled_subgraph.get_nodes().clone(),
                relabeled_subgraph.network.get_edges(),
            ),
            incoming_edges,
            outgoing_edges,
        );
        let mut new_mappings = self.mappings.clone();
        new_mappings.remove(&id);
        for (id, m) in relabeled_subgraph.mappings.iter() {
            new_mappings.insert(*id,*m);
        }
        let new_nodes = self.network.nodes
                                                    .iter()
                                                    .filter(|x| **x != id)
                                                    .cloned()
                                                    .collect::<BTreeSet<u32>>()
                                                    .union(&relabeled_subgraph.get_nodes())
                                                    .cloned()
                                                    .collect();
        HTN::new(new_nodes, new_graph.get_edges(), self.domain.clone(), new_mappings )
    }

    pub fn change_mappings(&mut self, changes: Vec<(u32, u32)>) {
        for (node_id, new_task_id) in changes {
            match self.mappings.remove(&node_id) {
                Some(_) => {
                    self.mappings.insert(node_id, new_task_id);
                },
                None => {panic!("Node not in the network")}
            }
        }
    }

    pub fn relabel_nodes(tn: &HTN, start_index: u32) -> HTN {
        let new_ids: HashMap<u32, u32> = tn.network.nodes.iter().cloned().zip(start_index..).collect();
        let mut subgraph = tn.network.clone();
        subgraph.change_ids(&new_ids);
        let new_mappings: HashMap<u32, u32> = tn.mappings.iter().map(|(k, v)| {
            match new_ids.get(k) {
                Some(new_id) => {
                    (*new_id, *v)
                }
                None => {
                    panic!("ID is not present")
                }
            }
        }).collect();
        HTN { network: subgraph, domain: tn.domain.clone(), mappings: new_mappings }
    }

    pub fn get_all_task_mappings(&self) -> Vec<u32>{
        self.mappings.iter().map(|(node, task)| {
            *task
        }).collect()
    }

    // pub fn is_isomorphic(tn1: &HTN, tn2: &HTN) -> bool {
    //     let layers_1 = tn1.network.to_layers();
    //     let layers_2 = tn2.network.to_layers();
    //     if layers_1.len() != layers_2.len() {
    //         return false;
    //     }
    //     let tasks_1 = tn1.layers_to_tasks(layers_1);
    //     let tasks_2 = tn2.layers_to_tasks(layers_2);

    //     for (x, y) in tasks_1.into_iter().zip(tasks_2.into_iter()) {
    //         if x != y {
    //             return false;
    //         }
    //     }

    //     return true;
    // }

    pub fn apply_action(&self, id: u32) -> HTN {
        if !self.is_primitive(id) {
            panic!("Task is not primitive")
        }
        let mut new_mapping = self.mappings.clone();
        new_mapping.remove(&id);
        let new_graph = self.network.remove_node(id);
        HTN { network: new_graph, mappings: new_mapping, domain: self.domain.clone()}
    }

    // fn layers_to_tasks(&self, layers: Vec<HashSet<u32>>) -> Vec<HashSet<&Task>> {
    //     let mut result = Vec::with_capacity(layers.len());
    //     for layer in layers.into_iter() {
    //         let tasks = layer.into_iter().map(|x| {
    //             let task_id = self.mappings.get(&x).unwrap();
    //             self.get_task(*task_id)
    //         });
    //         result.push(tasks.collect());
    //     }
    //     result
    // }

    pub fn is_primitive(&self, id: u32) -> bool {
        if !self.mappings.contains_key(&id) {
            panic!("id not in network");
        }
        match *self.get_task(id).borrow() {
            Task::Primitive(_) => true,
            _ => false
        }
    }

    // given a set U, separate ids into tuple (compound, primitive)
    pub fn separate_tasks(&self, tasks: &BTreeSet<u32>)
    -> (BTreeSet<u32>, BTreeSet<u32>) {
        let mut u_c = BTreeSet::new();
        let mut u_a = BTreeSet::new();
        for t in tasks.iter() {
            if self.is_primitive(*t) {
                u_a.insert(*t);
            } else {
                u_c.insert(*t);
            }
        }
        (u_c, u_a)
    }

    pub fn get_nodes(&self) -> &BTreeSet<u32> {
        &self.network.nodes
    }
    
    pub fn contains_task(&self, name: &str) -> bool {
        for (task_id, _) in self.mappings.iter() {
            if self.get_task(*task_id).borrow().get_name() == name {
                return true;
            }
        }
        return false;
    }

    pub fn change_domain(&mut self, new_domain: Rc<DomainTasks>) {
        self.domain = new_domain;
    }
    
}

impl fmt::Display for HTN {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(f, "digraph g {{");
        for node in self.get_nodes().iter() {
            writeln!(f, "\t{} [label={}];", *node, self.get_task(*node).borrow().get_name());
        }
        for (i,j) in self.network.get_edges() {
            writeln!(f, "\t{}->{};", i, j);
        }
        writeln!(f, "}}");
        Ok(())
     }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::domain_description::{DomainTasks, FONDProblem, Facts};

    use super::*;

    fn create_initial_tasks() -> DomainTasks {
        let empty = HashSet::new();
        let t1 = Task::Primitive(PrimitiveAction::new(
            "ObtainPermit".to_string(),
            1,
            empty.clone(),
            vec![empty.clone(),],
            vec![empty.clone(),],
        ));
        let t2 = Task::Primitive(PrimitiveAction::new(
            "HireBuilder".to_string(),
            1,
            empty.clone(),
            vec![empty.clone(),],
            vec![empty.clone(),],
        ));
        let t3 = Task::Compound(CompoundTask::new("Construct".to_string(), Vec::new()));
        let t4 = Task::Primitive(PrimitiveAction::new(
            "PayBuilder".to_string(),
            1,
            empty.clone(),
            vec![empty.clone(),],
            vec![empty.clone(),],
        ));
        DomainTasks::new(vec![t1,t2,t3,t4])
    }

    #[test]
    fn instantiation() {
        let t: BTreeSet<u32> = BTreeSet::from([1, 2, 3, 4]);
        let domain = Rc::new(create_initial_tasks());
        let alpha = HashMap::from([(1, 0), (2, 1), (3, 2), (4, 3)]);
        let orderings = Vec::from([(1, 3), (2, 3), (3, 4)]);
        let network = HTN::new(t, orderings, domain.clone(), alpha);
        assert_eq!(network.count_tasks(), 4);
        assert_eq!(network.get_task(1), domain.get_task(0));
        assert_eq!(network.get_task(2), domain.get_task(1));
        assert_eq!(network.get_task(3), domain.get_task(2));
        assert_eq!(network.get_task(4), domain.get_task(3));
    }

    fn decomposition_tasks() -> (
        Task,
        Task,
        Task,
        Task,
        Task,
    ) {
        let empty = HashSet::new();
        let t1 = Task::Primitive(PrimitiveAction::new(
            "BuildFoundation".to_string(),
            1,
            empty.clone(),
            vec![empty.clone(),],
            vec![empty.clone(),],
        ));
        let t2 = Task::Primitive(PrimitiveAction::new(
            "BuildFrame".to_string(),
            1,
            empty.clone(),
            vec![empty.clone(),],
            vec![empty.clone(),],
        ));
        let t3 = Task::Primitive(PrimitiveAction::new(
            "BuildRoof".to_string(),
            1,
            empty.clone(),
            vec![empty.clone(),],
            vec![empty.clone(),],
        ));
        let t4 = Task::Primitive(PrimitiveAction::new(
            "BuildWalls".to_string(),
            1,
            empty.clone(),
            vec![empty.clone(),],
            vec![empty.clone(),],
        ));
        let t5 = Task::Primitive(PrimitiveAction::new(
            "BuildInterior".to_string(),
            1,
            empty.clone(),
            vec![empty.clone(),],
            vec![empty.clone(),],
        ));
        (t1, t2, t3, t4, t5)
    }

    #[test]
    fn unconstrained_tasks_test() {
        let t: BTreeSet<u32> = BTreeSet::from([1, 2, 3, 4]);
        let domain = Rc::new(create_initial_tasks());
        let alpha = HashMap::from([(1, 1), (2, 2), (3, 3), (4, 4)]);
        let orderings: Vec<(u32, u32)> = Vec::from([(1, 3), (2, 3), (3, 4)]);
        let network = HTN::new(t, orderings, domain.clone(), alpha);
        let unconstrained = network.get_unconstrained_tasks();
        assert_eq!(unconstrained, BTreeSet::from([1, 2]));
    }

    #[test]
    fn decomposition_test() {
        let t: BTreeSet<u32> = BTreeSet::from([1, 2, 3, 4]);
        let mut domain = create_initial_tasks();
        let (t5, t6, t7, t8, t9) = decomposition_tasks();
        domain.add_task(t5.clone());
        domain.add_task(t6.clone());
        domain.add_task(t7.clone());
        domain.add_task(t8.clone());
        domain.add_task(t9.clone());
        let domain = Rc::new(domain);
        let t3_method = Method::new(
            "method-01".to_string(),
            HTN::new(
                BTreeSet::from([1, 2, 3, 4, 5]),
                Vec::from([(1, 2), (2, 3), (2, 4), (3, 5), (4, 5)]),
                domain.clone(),
                HashMap::from(
                    [(1, domain.get_id(&t5.get_name())), (2, domain.get_id(&t6.get_name())),
                    (3, domain.get_id(&t7.get_name())), (4, domain.get_id(&t8.get_name())),
                    (5, domain.get_id(&t9.get_name()))]
                )
            ),
        );
       let domain = domain.add_methods(vec![(domain.get_id("Construct"), t3_method.clone())]);
       let alpha = HashMap::from([(1, 0), (2, 1), (3, 2), (4, 3)]);
       let orderings: Vec<(u32, u32)> = Vec::from([(1, 3), (2, 3), (3, 4)]);
       let network = HTN::new(t, orderings, domain.clone(), alpha);
       let c_task = domain.get_task(domain.get_id("Construct"));
       if let Task::Compound(CompoundTask { name, methods }) = &*c_task.borrow() {
            let result = network.decompose(3, &methods[0]);
            assert_eq!(result.count_tasks(), 8);
            assert_eq!(result.get_unconstrained_tasks(), BTreeSet::from([1, 2]));
            assert_eq!(Graph::convert_edges_to_vec(&result.network.edges).len(), 8);
            assert_eq!(result.contains_task("Construct"), false);
            assert_eq!(result.network.edges.get(&1).unwrap().len(), 1);
       }
       else {
        panic!()
       };
    }

    #[test]
    // pub fn isomorphism_test() {
    //     let domain = Rc::new(create_initial_tasks());
    //     // first graph
    //     let nodes1: BTreeSet<u32> = BTreeSet::from([1, 2, 3, 4]);
    //     let orderings1: Vec<(u32, u32)> = Vec::from([(1, 3), (2, 3), (3, 4)]);
    //     let alpha =
    //     HashMap::from([(1, 1), (2, 2), (3, 3), (4, 4)]);
    //     let htn1 = HTN::new(
    //         nodes1,
    //         orderings1,
    //         domain.clone(),
    //         alpha,
    //     );

    //     let domain2 = Rc::new(create_initial_tasks());
    //     // second graph
    //     let nodes2: BTreeSet<u32> = BTreeSet::from([5, 6, 7, 8]);
    //     let orderings2: Vec<(u32, u32)> = Vec::from([(5, 7), (6, 7), (7, 8)]);
    //     let htn2 = HTN::new(
    //         nodes2,
    //         orderings2,
    //         domain2.clone(),
    //         HashMap::from([(5, 1), (6, 2), (7, 3), (8, 4)]),
    //     );

    //     let result = HTN::is_isomorphic(&htn1, &htn2);
    //     assert_eq!(result, true);
    // }

    #[test]
    pub fn is_primitive_test() {
        let domain = Rc::new(create_initial_tasks());
        // first graph
        let nodes1: BTreeSet<u32> = BTreeSet::from([1, 2, 3, 4]);
        let orderings1: Vec<(u32, u32)> = Vec::from([(1, 3), (2, 3), (3, 4)]);
        let alpha = HashMap::from([(1, 0), (2, 1), (3, 2), (4, 3)]);
        let htn = HTN::new(
            nodes1,
            orderings1,
            domain.clone(),
            alpha,
        );
        assert_eq!(htn.is_primitive(1), true);
        assert_eq!(htn.is_primitive(2), true);
        assert_eq!(htn.is_primitive(3), false);
        assert_eq!(htn.is_primitive(4), true);
    }

    #[test]
    pub fn apply_action_test() {
        let domain = Rc::new(create_initial_tasks());
        // first graph
        let nodes1: BTreeSet<u32> = BTreeSet::from([1, 2, 3, 4]);
        let orderings1: Vec<(u32, u32)> = Vec::from([(1, 3), (2, 3), (3, 4)]);
        let alpha = HashMap::from([(1, 0), (2, 1), (3, 2), (4, 3)]);
        let htn = HTN::new(
            nodes1,
            orderings1,
            domain.clone(),
            alpha,
        );
        let new_htn = htn.apply_action(2);
        assert_eq!(new_htn.count_tasks(), 3);
        assert_eq!(new_htn.contains_task("HireBuilder"), false);
        assert_eq!(new_htn.is_primitive(3), false);
        assert_eq!(new_htn.mappings.contains_key(&2), false);
        let new_htn_2 = new_htn.apply_action(1);
        assert_eq!(new_htn_2.get_nodes(), &BTreeSet::from([3,4]));
        assert_eq!(new_htn_2.is_primitive(3), false);
        assert_eq!(new_htn_2.mappings.contains_key(&1), false);
    }

    #[test]
    pub fn last_action_test() {
        let domain = Rc::new(create_initial_tasks());
        // first graph
        let nodes1: BTreeSet<u32> = BTreeSet::from([1, 2, 4]);
        let orderings1: Vec<(u32, u32)> = Vec::from([(1, 4), (2, 4)]);
        let alpha =
        HashMap::from([(1, 0), (2, 1), (4, 3)]);
        let htn = HTN::new(
            nodes1,
            orderings1,
            domain.clone(),
            alpha,
        );
        let new_htn = htn.apply_action(2);
        let new_htn_2 = new_htn.apply_action(1);
        let new_htn_3 = new_htn_2.apply_action(4);
        assert_eq!(new_htn_3.count_tasks(), 0);
    }

    #[test]
    pub fn is_empty() {
        let domain = Rc::new(create_initial_tasks());
        let nodes: BTreeSet<u32> = BTreeSet::from([1, 2, 4]);
        let orderings: Vec<(u32, u32)> = Vec::from([(1, 4), (2, 4)]);
        let alpha =
        HashMap::from([(1, 1), (2, 2), (4, 4)]);
        let htn = HTN::new(
            nodes,
            orderings,
            domain.clone(),
            alpha,
        );
        assert_eq!(htn.is_empty(), false);

        let empty_htn = HTN::new(
            BTreeSet::new(),
            Vec::new(),
            domain.clone(),
            HashMap::new(),
        );
        assert_eq!(empty_htn.is_empty(), true);
    }

    #[test]
    pub fn separate_tasks_test() {
        let t: BTreeSet<u32> = BTreeSet::from([1, 2, 3, 4]);
        let domain= Rc::new(create_initial_tasks());
        let alpha = HashMap::from([(1, 0), (2, 1), (3, 2), (4, 3)]);
        let orderings: Vec<(u32, u32)> = vec![];
        let network = HTN::new(t.clone(), orderings, domain.clone(), alpha);
        assert_eq!(
            network.separate_tasks(&t),
            (BTreeSet::from([3]), BTreeSet::from([1,2,4]))
        );
    }

    #[test]
    pub fn relabel_test() {
        let t: BTreeSet<u32> = BTreeSet::from([1, 2, 3, 4]);
        let mut domain = Rc::new(create_initial_tasks());
        let alpha = HashMap::from([(1, 0), (2, 1), (3, 2), (4, 3)]);
        let orderings: Vec<(u32, u32)> = vec![];
        let network = HTN::new(t.clone(), orderings, domain.clone(), alpha);
        let new_tn = HTN::relabel_nodes(&network, 2);
        assert_eq!(new_tn.get_nodes().len(), 4);
        assert_eq!(new_tn.get_task(2).borrow().get_name(), format!("ObtainPermit"));
        assert_eq!(new_tn.get_task(3).borrow().get_name(), format!("HireBuilder"));
        assert_eq!(new_tn.get_task(4).borrow().get_name(), format!("Construct"));
        assert_eq!(new_tn.get_task(5).borrow().get_name(), format!("PayBuilder"));
    }

    #[test]
    pub fn task_occurances_test() {
        let t: BTreeSet<u32> = BTreeSet::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let mut domain = Rc::new(create_initial_tasks());
        let alpha = HashMap::from(
            [(1, 0), (2, 1), (3, 2), (4, 3), (5,3), (6,3), (7,1), (8,2), (9,0)]
        );
        let orderings: Vec<(u32, u32)> = vec![(1,9), (2,1), (5,6), (6,7)];
        let network = HTN::new(t.clone(), orderings, domain.clone(), alpha);
        let occurances = network.count_tasks_with_frequency();
        assert_eq!(occurances.len(), 4);
        assert_eq!(*occurances.get(&0).unwrap(), 2);
        assert_eq!(*occurances.get(&1).unwrap(), 2);
        assert_eq!(*occurances.get(&2).unwrap(), 2);
        assert_eq!(*occurances.get(&3).unwrap(), 3);

    }

    #[test]
    pub fn collapse_tn_test() {
        let t: BTreeSet<u32> = BTreeSet::from([1, 2, 3, 4]);
        let mut domain = Rc::new(create_initial_tasks());
        let alpha = HashMap::from([(1, 0), (2, 1), (3, 2), (4, 3)]);
        let orderings: Vec<(u32, u32)> = vec![];
        let network = HTN::new(t.clone(), orderings, domain.clone(), alpha);
        let mut problem = FONDProblem {
            facts: Facts::new(vec![]),
            tasks: domain,
            initial_state: HashSet::new(),
            init_tn: network
        };
        problem.collapse_tn();
        assert_eq!(problem.init_tn.count_tasks(), 1);
        assert_eq!(problem.init_tn.get_unconstrained_tasks(), BTreeSet::from([1]));
    }

    #[test]
    pub fn recursive_decomposition_test() {
        let t = Task::Compound(CompoundTask { name: "recursive".to_owned(), methods: vec![] });
        let mut domain = Rc::new(DomainTasks::new(vec![t]));
        let domain = domain.add_methods(vec![
            (0,
            Method::new(
                "m1".to_owned(), HTN::new(
                BTreeSet::from([1, 2]),
                vec![],
                domain.clone(),
                HashMap::from([(1, 0), (2, 0)])
                )))
        ]);
        let tn = HTN::new(
            BTreeSet::from([1]),
            vec![],
            domain.clone(),
            HashMap::from([(1,0)])
        );
        match &*domain.get_task(0).borrow() {
            Task::Compound(CompoundTask { name: _, methods }) => {
                assert_eq!(methods.len(), 1);
                let new_tn = tn.decompose(1, &methods[0]);
                assert_eq!(new_tn.count_tasks(), 2);
                for t in new_tn.get_all_tasks() {
                    match &*t.borrow() {
                        Task::Compound(CompoundTask { name, methods }) => {
                            assert_eq!(name, "recursive");
                            assert_eq!(methods.len(), 1);
                            for (_, v) in new_tn.mappings.iter() {
                                assert_eq!(*v, 0);
                            }
                        },
                        Task::Primitive(_) => {}
                    }
                }
                
            },
            _ => {panic!("wrong task")}
        };
    }
}
