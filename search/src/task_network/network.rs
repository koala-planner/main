use super::Graph;
use super::task_structs::{CompoundTask, Method, PrimitiveAction, Task};
use std::collections::{HashMap, HashSet};
use std::fmt::{self, write};
use rand::distributions::DistString;
use std::hash::Hash;
use std::rc::Rc;
use rand::{distributions::Alphanumeric, Rng};

#[derive(Debug, Clone)]
pub struct HTN{
    network: Graph,
    // A mapping from task id in the network to its ID in the domain
    mappings: HashMap<u32, Rc<Task>>,
}

impl HTN {
    pub fn new(
        tasks: HashSet<u32>,
        orderings: Vec<(u32, u32)>,
        mappings: HashMap<u32, Rc<Task>>,
    ) -> HTN {
        HTN {
            network: Graph::new(tasks, orderings),
            mappings,
        }
    }

    pub fn get_orderings(&self) -> Vec<(u32,u32)>{
        self.network.get_edges()
    }

    pub fn get_all_tasks(&self) -> Vec<Rc<Task>> {
        self.network.nodes.iter().map(|id| {
            self.get_task(*id).unwrap()
        }).collect()
    }

    pub fn get_all_tasks_with_ids(&self) -> Vec<(Rc<Task>, u32)> {
        self.network.nodes.iter().map(|id| {
            (self.get_task(*id).unwrap(), *id)
        }).collect()
    }

    pub fn count_tasks(&self) -> usize {
        self.network.count_nodes()
    }

    pub fn is_empty(&self) -> bool {
        self.network.count_nodes() == 0
    }

    pub fn get_task(&self, id: u32) -> Option<Rc<Task>> {
        match self.mappings.get_key_value(&id) {
            Some((_, y)) => Some(Rc::clone(y)),
            None => None,
        }
    }

    pub fn get_unconstrained_tasks(&self) -> HashSet<u32> {
        self.network.get_unconstrained_nodes()
    }

    pub fn get_incoming_edges(&self, id: u32) -> HashSet<u32> {
        self.network.get_incoming_edges(id)
    }

    pub fn decompose(&self, id: u32, method: &Method) -> HTN {
        // TODO: Refactor this function
        let mut subgraph_nodes = method.decomposition.network.nodes.clone();
        let mut subgraph_edges = method.decomposition.network.edges.clone();
        let mut subgraph_mappings = method.decomposition.mappings.clone();
        if !subgraph_nodes.is_disjoint(&self.network.nodes) {
            let intersection: HashSet<u32> =
                method.decomposition.mappings.keys().cloned().collect();
            let network_max_id = self.network.nodes.iter().fold(u32::MIN, |a, b| a.max(*b));
            let max_id = subgraph_nodes.iter().fold(network_max_id, |a, b| a.max(*b));
            let new_ids: HashMap<u32, u32> = intersection.into_iter().zip(max_id + 1..).collect();
            for (prev_id, new_id) in new_ids.iter() {
                let mapping_val = subgraph_mappings.remove(&prev_id).unwrap();
                subgraph_mappings.insert(*new_id, mapping_val);
                if subgraph_edges.contains_key(&prev_id) {
                    let edges: HashSet<u32> = subgraph_edges.remove(&prev_id).unwrap();
                    let mapped_edges = edges.into_iter().map(|x| {
                        if new_ids.contains_key(&x) {
                            *new_ids.get(&x).unwrap()
                        } else {
                            x
                        }
                    });

                    subgraph_edges.insert(*new_id, mapped_edges.collect());
                }
                subgraph_nodes.remove(&prev_id);
                subgraph_nodes.insert(*new_id);
            }
        }
        let mut new_graph = self.network.clone();
        let outgoing_edges = self.network.get_outgoing_edges(id);
        let incoming_edges = self.network.get_incoming_edges(id);

        new_graph = new_graph.remove_node(id);
        new_graph = new_graph.add_subgraph(
            Graph::new(
                subgraph_nodes.clone(),
                Graph::convert_edges_to_vec(&subgraph_edges),
            ),
            incoming_edges,
            outgoing_edges,
        );

        let mut new_mappings = self.mappings.clone();
        new_mappings.remove(&id);
        for (id, m) in subgraph_mappings {
            new_mappings.insert(id, m);
        }
        let new_nodes = self
            .network
            .nodes
            .iter()
            .filter(|x| **x != id)
            .cloned()
            .collect::<HashSet<u32>>()
            .union(&subgraph_nodes)
            .cloned()
            .collect();
        HTN::new(new_nodes, new_graph.get_edges(), new_mappings)
    }

    pub fn is_isomorphic(tn1: &HTN, tn2: &HTN) -> bool {
        let layers_1 = tn1.network.to_layers();
        let layers_2 = tn2.network.to_layers();
        if layers_1.len() != layers_2.len() {
            return false;
        }
        let tasks_1 = tn1.layers_to_tasks(layers_1);
        let tasks_2 = tn2.layers_to_tasks(layers_2);

        for (x, y) in tasks_1.into_iter().zip(tasks_2.into_iter()) {
            if x != y {
                return false;
            }
        }

        return true;
    }

    pub fn apply_action(&self, id: u32) -> HTN {
        let mut new_mapping = self.mappings.clone();
        new_mapping.remove(&id);
        let new_graph = self.network.remove_node(id);
        HTN { network: new_graph, mappings: new_mapping }
    }

    fn layers_to_tasks(&self, layers: Vec<HashSet<u32>>) -> Vec<HashSet<&Task>> {
        let mut result = Vec::with_capacity(layers.len());
        for layer in layers.into_iter() {
            let tasks = layer.into_iter().map(|x| self.mappings.get(&x).unwrap().as_ref());
            result.push(tasks.collect());
        }
        result
    }

    pub fn is_primitive(&self, id: u32) -> bool {
        let task = self.mappings.get(&id).unwrap();
        match task.as_ref() {
            Task::Primitive(_) => true,
            _ => false
        }
    }

    // given a set U, separate ids into tuple (compound, primitive)
    pub fn separate_tasks(&self, tasks: &HashSet<u32>)
    -> (HashSet<u32>, HashSet<u32>) {
        let mut u_c = HashSet::new();
        let mut u_a = HashSet::new();
        for t in tasks.iter() {
            if self.is_primitive(*t) {
                u_a.insert(*t);
            } else {
                u_c.insert(*t);
            }
        }
        (u_c, u_a)
    }

    // collapses all tasks in the network into one abstract task
    pub fn collapse_tn(&self) -> HTN {
        let rand_s: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
        let new_task = Task::Compound(CompoundTask {
            // Planner Generated Task
            name: "__P_G_T_".to_string() + &rand_s,
            methods: vec![Method::new(
                "__P_G_M".to_string(),
                self.clone()
            )]
        });
        let max_id = (*self.network.nodes.iter().max().unwrap()) + 1;
        let mut new_mappings = self.mappings.clone();
        new_mappings.insert(max_id, Rc::new(new_task));
        HTN::new(
            HashSet::from([max_id]), 
            vec![], 
            new_mappings
        )
    }

    pub fn get_nodes(&self) -> &HashSet<u32> {
        &self.network.nodes
    }

    pub fn change_mappings(&self, new_mappings: HashMap<u32, Rc<Task>>) -> HTN {
        HTN { network: self.network.clone(), mappings: new_mappings }
    }

    pub fn change_task(&mut self, id: u32, new_task: Rc<Task>) {
        self.mappings.remove(&id);
        self.mappings.insert(id, new_task);
    }

    pub fn contains_task(&self, name: &str) -> bool {
        for (_, task) in self.mappings.iter() {
            if task.get_name() == name {
                return true;
            }
        }
        return false;
    }
    
}

impl fmt::Display for HTN {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let layers = self.network.to_layers();
        let layers = self.layers_to_tasks(layers);
        for (i, layer) in layers.iter().enumerate() {
            write!(f, "layer {}:\n", i);
            for task in layer.iter() {
                write!(f, "\t{}\n", task);
            }
        }
        Ok(())
     }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_initial_tasks() -> (Rc<Task>, Rc<Task>, Rc<Task>, Rc<Task>) {
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
        let (t1, t2, t3, t4) = (Rc::new(t1), Rc::new(t2), Rc::new(t3), Rc::new(t4));
        (t1, t2, t3, t4)
    }

    #[test]
    fn instantiation() {
        let t: HashSet<u32> = HashSet::from([1, 2, 3, 4]);
        let (t1, t2, t3, t4) = create_initial_tasks();
        let alpha =
            HashMap::from([(1, Rc::clone(&t1)), (2, Rc::clone(&t2)), (3, Rc::clone(&t3)), (4, Rc::clone(&t4))]);
        let orderings: Vec<(u32, u32)> = Vec::from([(1, 3), (2, 3), (3, 4)]);
        let network = HTN::new(t, orderings, alpha);
        assert_eq!(network.count_tasks(), 4);
        assert_eq!(network.get_task(1).unwrap(), t1);
        assert_eq!(network.get_task(2).unwrap(), t2);
        assert_eq!(network.get_task(3).unwrap(), t3);
        assert_eq!(network.get_task(4).unwrap(), t4);
        assert_eq!(network.get_task(5), None);
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
        let t: HashSet<u32> = HashSet::from([1, 2, 3, 4]);
        let (t1, t2, t3, t4) = create_initial_tasks();
        let alpha =
            HashMap::from([(1, t1), (2, t2), (3, t3), (4, t4)]);
        let orderings: Vec<(u32, u32)> = Vec::from([(1, 3), (2, 3), (3, 4)]);
        let network = HTN::new(t, orderings, alpha);
        let unconstrained = network.get_unconstrained_tasks();
        assert_eq!(unconstrained, HashSet::from([1, 2]));
    }

    #[test]
    fn decomposition_test() {
        let t: HashSet<u32> = HashSet::from([1, 2, 3, 4]);
        let (t1, t2, t3, t4) = create_initial_tasks();
        let (t5, t6, t7, t8, t9) = decomposition_tasks();
        let t3_method = Method::new(
            "method-01".to_string(),
            HTN::new(
                HashSet::from([1, 2, 3, 4, 5]),
                Vec::from([(1, 2), (2, 3), (2, 4), (3, 5), (4, 5)]),
                HashMap::from(
                    [(1, Rc::new(t5)), (2, Rc::new(t6)), (3, Rc::new(t7)), (4, Rc::new(t8)), (5, Rc::new(t9))]
                ),
            ),
        );
        let alpha = HashMap::from([(1, t1), (2, t2), (3, t3), (4, t4)]);
        let orderings: Vec<(u32, u32)> = Vec::from([(1, 3), (2, 3), (3, 4)]);
        let network = HTN::new(t, orderings, alpha);
        let result = network.decompose(3, &t3_method);
        assert_eq!(result.count_tasks(), 8);
        assert_eq!(result.get_unconstrained_tasks(), HashSet::from([1, 2]));
        assert_eq!(Graph::convert_edges_to_vec(&result.network.edges).len(), 8);
        assert_eq!(result.get_task(3), None);
        assert_eq!(result.network.edges.get(&1).unwrap().len(), 1);
    }

    #[test]
    pub fn isomorphism_test() {
        let (t1, t2, t3, t4) = create_initial_tasks();
        // first graph
        let nodes1: HashSet<u32> = HashSet::from([1, 2, 3, 4]);
        let orderings1: Vec<(u32, u32)> = Vec::from([(1, 3), (2, 3), (3, 4)]);
        let alpha =
        HashMap::from([(1, t1), (2, t2), (3, t3), (4, t4)]);
        let htn1 = HTN::new(
            nodes1,
            orderings1,
            alpha,
        );

        let (t5, t6, t7, t8) = create_initial_tasks();
        // second graph
        let nodes2: HashSet<u32> = HashSet::from([5, 6, 7, 8]);
        let orderings2: Vec<(u32, u32)> = Vec::from([(5, 7), (6, 7), (7, 8)]);
        let htn2 = HTN::new(
            nodes2,
            orderings2,
            HashMap::from([(5, t5), (6, t6), (7, t7), (8, t8)]),
        );

        let result = HTN::is_isomorphic(&htn1, &htn2);
        assert_eq!(result, true);
    }

    #[test]
    pub fn is_primitive_test() {
        let (t1, t2, t3, t4) = create_initial_tasks();
        // first graph
        let nodes1: HashSet<u32> = HashSet::from([1, 2, 3, 4]);
        let orderings1: Vec<(u32, u32)> = Vec::from([(1, 3), (2, 3), (3, 4)]);
        let alpha =
        HashMap::from([(1, t1), (2, t2), (3, t3), (4, t4)]);
        let htn = HTN::new(
            nodes1,
            orderings1,
            alpha,
        );
        assert_eq!(htn.is_primitive(1), true);
        assert_eq!(htn.is_primitive(2), true);
        assert_eq!(htn.is_primitive(3), false);
        assert_eq!(htn.is_primitive(4), true);
    }

    #[test]
    pub fn apply_action_test() {
        let (t1, t2, t3, t4) = create_initial_tasks();
        // first graph
        let nodes1: HashSet<u32> = HashSet::from([1, 2, 3, 4]);
        let orderings1: Vec<(u32, u32)> = Vec::from([(1, 3), (2, 3), (3, 4)]);
        let alpha =
        HashMap::from([(1, t1), (2, t2), (3, t3), (4, t4)]);
        let htn = HTN::new(
            nodes1,
            orderings1,
            alpha,
        );
        let new_htn = htn.apply_action(2);
        assert_eq!(new_htn.count_tasks(), 3);
        assert_eq!(new_htn.get_task(2), None);
        assert_eq!(new_htn.is_primitive(3), false);
        assert_eq!(new_htn.mappings.contains_key(&2), false);
        let new_htn_2 = new_htn.apply_action(1);
        assert_eq!(new_htn_2.count_tasks(), 2);
        assert_eq!(new_htn_2.get_task(1), None);
        assert_eq!(new_htn_2.is_primitive(3), false);
        assert_eq!(new_htn_2.mappings.contains_key(&1), false);
    }

    #[test]
    pub fn last_action_test() {
        let (t1, t2, t3, t4) = create_initial_tasks();
        // first graph
        let nodes1: HashSet<u32> = HashSet::from([1, 2, 4]);
        let orderings1: Vec<(u32, u32)> = Vec::from([(1, 4), (2, 4)]);
        let alpha =
        HashMap::from([(1, t1), (2, t2), (4, t4)]);
        let htn = HTN::new(
            nodes1,
            orderings1,
            alpha,
        );
        let new_htn = htn.apply_action(2);
        let new_htn_2 = new_htn.apply_action(1);
        let new_htn_3 = new_htn_2.apply_action(4);
        assert_eq!(new_htn_3.count_tasks(), 0);
    }

    #[test]
    pub fn is_empty() {
        let (t1, t2, t3, t4) = create_initial_tasks();
        let nodes: HashSet<u32> = HashSet::from([1, 2, 4]);
        let orderings: Vec<(u32, u32)> = Vec::from([(1, 4), (2, 4)]);
        let alpha =
        HashMap::from([(1, t1), (2, t2), (4, t4)]);
        let htn = HTN::new(
            nodes,
            orderings,
            alpha,
        );
        assert_eq!(htn.is_empty(), false);

        let empty_htn = HTN::new(
            HashSet::new(),
            Vec::new(),
            HashMap::new(),
        );
        assert_eq!(empty_htn.is_empty(), true);
    }

    #[test]
    pub fn separate_tasks_test() {
        let t: HashSet<u32> = HashSet::from([1, 2, 3, 4]);
        let (t1, t2, t3, t4) = create_initial_tasks();
        let alpha =
            HashMap::from([(1, Rc::clone(&t1)), (2, Rc::clone(&t2)), (3, Rc::clone(&t3)), (4, Rc::clone(&t4))]);
        let orderings: Vec<(u32, u32)> = vec![];
        let network = HTN::new(t.clone(), orderings, alpha);
        assert_eq!(
            network.separate_tasks(&t),
            (HashSet::from([3]), HashSet::from([1,2,4]))
        );
    }

    #[test]
    pub fn collapse_tn_test() {
        let t: HashSet<u32> = HashSet::from([1, 2, 3, 4]);
        let (t1, t2, t3, t4) = create_initial_tasks();
        let alpha =
            HashMap::from([(1, Rc::clone(&t1)), (2, Rc::clone(&t2)), (3, Rc::clone(&t3)), (4, Rc::clone(&t4))]);
        let orderings: Vec<(u32, u32)> = vec![];
        let network = HTN::new(t.clone(), orderings, alpha);
        let new_tn = network.collapse_tn();
        assert_eq!(new_tn.count_tasks(), 1);
        assert_eq!(new_tn.get_unconstrained_tasks(), HashSet::from([5]));
    } 
}
