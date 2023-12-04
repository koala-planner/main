use std::{
    collections::{HashMap, HashSet, BTreeSet},
    iter::repeat,
};

#[derive(Debug, Clone)]
pub struct Graph {
    pub nodes: BTreeSet<u32>,
    pub edges: HashMap<u32, BTreeSet<u32>>,
}

impl Graph {
    pub fn new(nodes: BTreeSet<u32>, orderings: Vec<(u32, u32)>) -> Self {
        let mut edges: HashMap<u32, BTreeSet<u32>> = HashMap::with_capacity(nodes.len());
        for edge in orderings.into_iter() {
            match edge {
                (x, y) => match edges.get_mut(&x) {
                    Some(val) => {
                        val.insert(y);
                    }
                    None => {
                        edges.insert(x, BTreeSet::from([y]));
                    }
                },
            }
        }
        Graph {
            nodes,
            edges: edges,
        }
    }

    pub fn get_edges(&self) -> Vec<(u32, u32)> {
        self.edges
            .clone()
            .into_iter()
            .map(|(k, v)| repeat(k).zip(v).collect::<Vec<_>>())
            .flatten()
            .collect()
    }

    pub fn convert_edges_to_vec(edges: &HashMap<u32, BTreeSet<u32>>) -> Vec<(u32, u32)> {
        edges
            .clone()
            .into_iter()
            .map(|(k, v)| repeat(k).zip(v).collect::<Vec<_>>())
            .flatten()
            .collect()
    }

    pub fn count_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_neighbors(&self, id: u32) -> Option<&BTreeSet<u32>> {
        self.edges.get(&id)
    }

    pub fn get_unconstrained_nodes(&self) -> BTreeSet<u32> {
        let mut result = self.nodes.clone();
        for k in self.edges.keys() {
            for val in self.edges.get(k).unwrap() {
                result.remove(val);
            }
        }
        result
    }

    pub fn get_incoming_edges(&self, id: u32) -> BTreeSet<u32> {
        BTreeSet::from_iter(
            self.edges
                .iter()
                .filter(|(_, v)| v.contains(&id))
                .map(|(k, _)| *k),
        )
    }

    pub fn get_outgoing_edges(&self, id: u32) -> BTreeSet<u32> {
        match self.edges.get(&id) {
            Some(x) => x.clone(),
            None => BTreeSet::new()
        }
    }

    pub fn remove_node(&self, id: u32) -> Graph {
        if !self.nodes.contains(&id) {
            self.clone()
        } else {
            let mut new_nodes = self.nodes.clone();
            new_nodes.remove(&id);
            let mut new_edges = Vec::with_capacity(self.edges.keys().len());
            for (key, values) in self.edges.iter() {
                if *key == id {
                    continue;
                }
                for value in values {
                    if *value == id {
                        continue;
                    } else {
                        new_edges.push((*key, *value))
                    }
                }
            }
            Graph::new(new_nodes, new_edges)
        }
    }

    pub fn add_subgraph(
        &self,
        subgraph: Graph,
        incoming_edges: BTreeSet<u32>,
        outgoing_edges: BTreeSet<u32>,
    ) -> Graph {
        let nodes: BTreeSet<u32> = self.nodes.clone().union(&subgraph.nodes).cloned().collect();
        if !self.nodes.intersection(&subgraph.nodes).collect::<BTreeSet<&u32>>().is_empty() {
            panic!("The IDs of subgraph and current graph are not disjoint")
        }
        let mut orderings = self.edges.clone();

        // Adding incoming edges
        let unconstrained_nodes = subgraph.get_unconstrained_nodes();
        for node in incoming_edges.iter() {
            match orderings.contains_key(node) {
                false => orderings.insert(*node, unconstrained_nodes.clone()),
                true => orderings.insert(
                    *node,
                    unconstrained_nodes
                        .union(orderings.get(node).unwrap())
                        .cloned()
                        .collect(),
                ),
            };
        }

        // Adding outgoing edges
        let terminal_nodes: HashSet<u32> = subgraph
            .nodes
            .difference(&subgraph.edges.keys().cloned().collect())
            .cloned()
            .collect();
        for node in terminal_nodes.iter() {
            orderings.insert(*node, outgoing_edges.clone());
        }

        for (key, value) in subgraph.edges {
            orderings.insert(key, value);
        }

        let orderings = orderings
            .into_iter()
            .map(|(k, v)| repeat(k).zip(v).collect::<Vec<_>>())
            .flatten()
            .collect();
        Graph::new(nodes, orderings)
    }

    pub fn to_layers(&self) -> Vec<HashSet<u32>> {
        let mut result: Vec<HashSet<u32>> = Vec::new();
        let mut prev_layer = HashSet::from_iter(self.get_unconstrained_nodes().iter().cloned());
        result.push(prev_layer.clone());
        loop {
            let mut layer: HashSet<u32> = HashSet::new();
            for node in prev_layer.iter() {
                match self.edges.get(node) {
                    Some(x) => {
                        for outgoing in x.iter() {
                            layer.insert(*outgoing);
                        }
                    }
                    None => continue,
                }
            }
            if layer.is_empty() {
                break;
            }
            result.push(layer.clone());
            prev_layer = layer;
        }
        result
    }

    pub fn get_leaf_nodes(&self) -> HashSet<u32> {
        let mut leaves = self.nodes
                        .iter()
                        .filter(|x| !self.edges.contains_key(x))
                        .cloned()
                        .collect();
        leaves
    }

    pub fn add_node(&self,
        id: u32,
        incoming_edges: BTreeSet<u32>,
        outgoing_edges: BTreeSet<u32>
    ) -> Result<Graph, &str> {
        if self.nodes.contains(&id) {
            return Err("Node Already Exists")
        } else {
            let mut new_nodes = self.nodes.clone();
            new_nodes.insert(id);
            let mut new_edges = self.edges.clone();
            if !incoming_edges.is_empty() {
                for v1 in incoming_edges.iter(){
                    if new_edges.contains_key(v1) {
                        new_edges.get_mut(v1).unwrap().insert(id);
                    } else {
                        new_edges.insert(*v1, BTreeSet::from([id]));
                    }
                }
            }
            if !outgoing_edges.is_empty() {
                new_edges.insert(id, outgoing_edges);
            }
            Ok(Graph {
                nodes: new_nodes,
                edges: new_edges
            })
        }
    }

    // change IDs based on a vec of partial (i.e., not complete set of nodes) new_ids
    pub fn change_ids(&mut self, new_ids: &HashMap<u32,u32>) {
        let mut edges = HashMap::new();
        // remove previous nodes & edges
        for (prev_id, new_id) in new_ids.iter() {
            if !self.nodes.remove(&prev_id) {
                panic!("Node not in the graph");
            }
            if self.edges.contains_key(prev_id) {
                let mut processed_edges = self.edges.remove(prev_id).unwrap();
                processed_edges = processed_edges.iter().map(|x| {
                    if new_ids.contains_key(x) {
                        *new_ids.get(x).unwrap()
                    } else {
                        *x
                    }
                }).collect();
                edges.insert(*new_id, processed_edges);
            }
        }
        // add edges
        for (node, connections) in edges.into_iter() {
            self.edges.insert(node, connections);
        }
        // add new nodes 
        for (_, new_id) in new_ids.iter() {
            self.nodes.insert(*new_id);
        }
    }

    // Labeled isomorphism
    pub fn is_isomorphic(g1: &Graph, g2: &Graph, l1: HashMap<u32, u32>, l2: HashMap<u32, u32>) -> bool {
        // bijection
        let mut core_1 = HashMap::with_capacity(g1.count_nodes());
        let mut core_2 = HashMap::with_capacity(g2.count_nodes());
        // in-out terminals
        let in_1 = HashMap::with_capacity(g1.count_nodes());
        let in_2 = HashMap::with_capacity(g2.count_nodes());
        let out_1 = HashMap::with_capacity(g1.count_nodes());
        let out_2 = HashMap::with_capacity(g2.count_nodes());
        // state = depth
        let mut fringe: Vec<u32> = Vec::new();
        fringe.push(0);
        while !fringe.is_empty() {
            let state = fringe.pop().unwrap();
            // Compute P
            let mut p: Vec<(u32, u32)> = vec![];
            // TODO: check whether s should be bigger than "state" or 0
            // TODO: Exxplore what does min mean
            // image of in - out oin current state
            let in_1_s: Vec<u32> = in_1.iter()
                .filter(|(_, s)| **s > 0)
                .map(|(x, _)| *x).collect();
            let in_2_s: Vec<u32> = in_2.iter()
                .filter(|(_, s)| **s > 0)
                .map(|(x, _)| *x).collect();
            let out_1_s: Vec<u32> = out_1.iter()
                .filter(|(_, s)| **s > 0)
                .map(|(x, _)| *x).collect();
            let out_2_s: Vec<u32> = out_2.iter()
                .filter(|(_, s)| **s > 0)
                .map(|(x, _)| *x).collect();
            // rule based construction
            if (out_1_s.len() > 0) && (out_2_s.len() > 0) {
                for p1 in out_1_s.into_iter() {
                    for p2 in out_2_s.into_iter() {
                        if l1.get(&p1).unwrap() == l2.get(&p2).unwrap() {
                            p.push((p1, p2));
                        }
                    }
                }
            } else if (in_1_s.len() > 0) && (in_2_s.len() > 0) {
                for p1 in in_1_s.into_iter() {
                    for p2 in in_2_s.into_iter() {
                        if l1.get(&p1).unwrap() == l2.get(&p2).unwrap() {
                            p.push((p1, p2));
                        }
                    }
                }
            } else {
                let p1_list: Vec<u32> = g1.nodes.difference(&BTreeSet::from_iter(core_1
                        .iter().map(|(x, _)| *x))).cloned().collect();
                let p2_list: Vec<u32> = g2.nodes.difference(&BTreeSet::from_iter(core_2
                        .iter().map(|(x, _)| *x))).cloned().collect();
                for p1 in p1_list.into_iter() {
                    for p2 in p2_list.into_iter() {
                        if l1.get(&p1).unwrap() == l2.get(&p2).unwrap() {
                            p.push((p1, p2));
                        }
                    }
                }
            }
            for (n, m) in p.iter() {
                // assert same predecesssors
                let pred_n_list = g1.get_incoming_edges(*n);
                let pred_m_list = g1.get_incoming_edges(*m);
                if pred_n_list.len() != pred_m_list.len() {
                    continue;
                } else {
                    for pred_n in pred_n_list.iter() {
                        let mut is_feasible = false;
                        for pred_m in pred_m_list.iter() {
                            if l1.get(pred_n).unwrap() == l2.get(pred_m).unwrap() {
                                is_feasible = true;
                                break;
                            }
                        }
                        if is_feasible == false {
                            continue;
                        }
                    }
                }
                // assert same successors
                let succ_n_list = g1.get_outgoing_edges(*n);
                let succ_m_list = g1.get_outgoing_edges(*m);
                if succ_n_list.len() != succ_m_list.len() {
                    continue;
                } else {
                    for succ_n in succ_n_list.iter() {
                        let mut is_feasible = false;
                        for succ_m in succ_m_list.iter() {
                            if l1.get(succ_n).unwrap() == l2.get(succ_m).unwrap() {
                                is_feasible = true;
                                break;
                            }
                        }
                        if is_feasible == false {
                            continue;
                        }
                    }
                }
                core_1.insert(*n, state);
                core_2.insert(*m, state);
                fringe.push(state+1);
            }
        }
        false
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn instantiation() {
        let nodes: BTreeSet<u32> = BTreeSet::from([1, 2, 3, 4]);
        let orderings: Vec<(u32, u32)> = Vec::from([(1, 3), (2, 3), (3, 4)]);
        let g = Graph::new(nodes, orderings);
        assert_eq!(g.count_nodes(), 4);
        assert!(g.get_neighbors(1).unwrap().contains(&3));
        assert!(g.get_neighbors(2).unwrap().contains(&3));
        assert!(g.get_neighbors(3).unwrap().contains(&4));
    }

    #[test]
    fn unconstrained_nodes_test() {
        let nodes: BTreeSet<u32> = BTreeSet::from([1, 2, 3, 4]);
        let orderings: Vec<(u32, u32)> = Vec::from([(1, 3), (2, 3), (3, 4)]);
        let g = Graph::new(nodes, orderings);
        let unconstrained = g.get_unconstrained_nodes();
        assert_eq!(unconstrained, BTreeSet::from([1, 2]));
    }

    #[test]
    fn incoming_edges_test() {
        let nodes: BTreeSet<u32> = BTreeSet::from([1, 2, 3, 4]);
        let orderings: Vec<(u32, u32)> = Vec::from([(1, 3), (2, 3), (3, 4)]);
        let g = Graph::new(nodes, orderings);
        let result = g.get_incoming_edges(3);
        assert_eq!(result, BTreeSet::from([1, 2]))
    }

    #[test]
    fn delete_node_test() {
        let nodes: BTreeSet<u32> = BTreeSet::from([1, 2, 3, 4]);
        let orderings: Vec<(u32, u32)> = Vec::from([(1, 3), (2, 3), (3, 4)]);
        let g = Graph::new(nodes, orderings);
        let new_g = g.remove_node(3);
        let unconstrainted = new_g.get_unconstrained_nodes();
        assert_eq!(new_g.count_nodes(), 3);
        assert_eq!(unconstrainted, BTreeSet::from([1, 2, 4]))
    }

    #[test]
    fn add_subgraph_test() {
        let nodes: BTreeSet<u32> = BTreeSet::from([1, 2, 4]);
        let orderings: Vec<(u32, u32)> = Vec::from([]);
        let g = Graph::new(nodes, orderings);

        let subgraph_nodes = BTreeSet::from([5, 6, 7, 8, 9]);
        let subgraph_orderings: Vec<(u32, u32)> =
            Vec::from([(5, 6), (6, 7), (6, 8), (7, 9), (8, 9)]);
        let subgraph = Graph::new(subgraph_nodes, subgraph_orderings);

        let result = g.add_subgraph(subgraph, BTreeSet::from([1, 2]), BTreeSet::from([4]));

        // inherited orderings
        assert_eq!(*result.edges.get(&1).unwrap(), BTreeSet::from([5]));
        assert_eq!(*result.edges.get(&2).unwrap(), BTreeSet::from([5]));
        assert_eq!(*result.edges.get(&9).unwrap(), BTreeSet::from([4]));

        //subgraph orderings
        assert_eq!(*result.edges.get(&5).unwrap(), BTreeSet::from([6]));
        assert_eq!(*result.edges.get(&6).unwrap(), BTreeSet::from([7, 8]));
        assert_eq!(*result.edges.get(&7).unwrap(), BTreeSet::from([9]));
        assert_eq!(*result.edges.get(&8).unwrap(), BTreeSet::from([9]));
    }

    #[test]
    pub fn graph_to_layers_test() {
        let nodes: BTreeSet<u32> = BTreeSet::from([1, 2, 3, 4]);
        let orderings: Vec<(u32, u32)> = Vec::from([(1, 3), (2, 3), (3, 4)]);
        let g = Graph::new(nodes, orderings);
        let result = g.to_layers();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], HashSet::from([1, 2]));
        assert_eq!(result[1], HashSet::from([3]));
        assert_eq!(result[2], HashSet::from([4]));
    }

    #[test]
    pub fn leaf_nodes_test() {
        let nodes: BTreeSet<u32> = BTreeSet::from([1, 2, 3, 4, 5]);
        let orderings: Vec<(u32, u32)> = Vec::from([
            (1, 3), (2, 3), (3, 4), (3,5)
        ]);
        let g = Graph::new(nodes, orderings);
        let result = g.get_leaf_nodes();
        assert_eq!(result, HashSet::from([4,5]));
        let mut g2 = g.remove_node(5);
        g2 = g2.remove_node(4);
        let result = g2.get_leaf_nodes();
        assert_eq!(result, HashSet::from([3]));
    }

    #[test]
    pub fn insert_node_test() {
        let nodes: BTreeSet<u32> = BTreeSet::from([1, 2, 3, 4, 5]);
        let orderings: Vec<(u32, u32)> = Vec::from([
            (1, 3), (2, 3), (3, 4), (3,5)
        ]);
        let g = Graph::new(nodes, orderings);
        let result = g.add_node(
            6, BTreeSet::from([5,4]),
            BTreeSet::new()
        ).unwrap();
        assert_eq!(result.count_nodes(), 6);
        assert_eq!(result.get_incoming_edges(6), BTreeSet::from([5,4]));
    }
}
