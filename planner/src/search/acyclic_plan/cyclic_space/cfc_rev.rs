use std::{
    borrow::Borrow,
    collections::{BTreeSet, HashMap, HashSet, VecDeque},
};

use super::*;

pub struct CFCRevStar {
    search_graph: SearchGraph,
    found: BTreeSet<u32>,
    h_type: HeuristicType,
    revisables: BTreeSet<u32>,
    open: BTreeSet<u32>,
    prev_cost: HashMap<u32, f32>
}

impl CFCRevStar {
    pub fn new(problem: &FONDProblem, h_type: HeuristicType) -> CFCRevStar {
        CFCRevStar {
            search_graph: SearchGraph::new(problem),
            found: BTreeSet::new(),
            h_type,
            revisables: BTreeSet::new(),
            open: BTreeSet::new(),
            prev_cost: HashMap::new()
        }
    }

    pub fn search(&mut self) -> SearchResult {
        while !self.search_graph.is_terminated()
            && self
                .search_graph
                .ids
                .get(&self.search_graph.root)
                .unwrap()
                .borrow()
                .cost
                .is_finite()
        {
            // Node Expansion
            let n = self.search_graph.find_a_tip_node();
            self.found.insert(n);
            self.search_graph.expand(n, &self.h_type, true);

            // Initialize revisables
            self.build_revisable(n);
            self.revisables.iter().map(|x| {
                let mut node = self.search_graph.ids.get(x).unwrap().borrow_mut();
                self.prev_cost.insert(*x, node.cost.clone());
                node.cost = f32::INFINITY;
            });

            // initialize open nodes
            for (id, node) in self.search_graph.ids.iter() {
                let node = node.borrow();
                // is tip node?
                if node.is_terminal() || node.connections.is_none() {
                    self.open.insert(*id);
                } else {
                    self.found.remove(id);
                }
            }

            // definitive cost assignment
            while !self.open.is_empty() {
                let m = self.open.pop_first().unwrap();
                // TODO: if m is a non-tip or node
                let node_m = self.search_graph.ids.get(&m).unwrap().borrow();
                match &node_m.connections {
                    Some(connectors) => {
                        let mut or_nodes = HashSet::new();
                        for connector in connectors.children.iter() {
                            if connector.children.len() == 1 {
                                or_nodes.insert(connector.children.iter().next().unwrap().clone());
                            }
                        }
                        // TODO: vague cost assignment in the paper
                    }
                    None => {}
                }
                drop(node_m);
                self.cost_prop(m, &mut VecDeque::new());
            }
        }
        todo!()
    }

    fn build_revisable(&mut self, n: u32) {
        let mut revisable_set = BTreeSet::from([n]);
        let mut working_set = BTreeSet::from([n]);
        while !working_set.is_empty() {
            let m = working_set.pop_first().unwrap();
            let parent_nodes = &self.search_graph.ids.get(&m).unwrap().borrow().parents;
            match parent_nodes {
                Some(parents) => {
                    for p_id in parents {
                        let p = self.search_graph.ids.get(p_id).unwrap().borrow();
                        match p.get_marked_connection() {
                            Some(marked) => {
                                if marked.children.contains(&m) {
                                    working_set.insert(*p_id);
                                    revisable_set.insert(*p_id);
                                }
                            }
                            None => {}
                        }
                    }
                }
                None => {
                    revisable_set.insert(m);
                }
            }
        }
        self.revisables = revisable_set;
    }

    fn cost_prop(&mut self, m: u32, cost_prop_queue: VecDeque<u32>){
        self.found.insert(m);
        let mut m_node = &self.search_graph.ids.get(&m).unwrap().borrow_mut();
        // TODO: foreach p \in P(m)
        if let Some(parents) = &m_node.parents {
            for p in parents {
                if self.found.contains(&p) {
                    continue;
                }
                // TODO: if revisable
                if self.revisables.contains(&p) {
                    let mut p_node = self.search_graph.ids.get(&p).unwrap().borrow_mut();
                    if let Some(mut connectors) = p_node.connections.as_mut() {
                        // TODO: if all children are found
                        if self.all_children_found(&connectors) {
                            // TODO: if and/or or in open
                            let mut costs = Vec::new();
                            for (index, connector) in connectors.children.iter_mut().enumerate() {
                                let mut connector_cost = connector.cost;
                                let succ_cost: f32 = connector
                                    .children
                                    .iter()
                                    .map(|s_id| {
                                        // TODO: add heuristic instead of 0
                                        let f = self.search_graph
                                            .ids
                                            .get(s_id)
                                            .unwrap()
                                            .borrow()
                                            .cost
                                            .clone();
                                        let h = 0.0;
                                        if f > h {
                                            f
                                        } else {
                                            h
                                        }
                                    })
                                    .sum();
                                // children status
                                let all_solved = connector
                                    .children
                                    .iter()
                                    .map(|s_id| self.search_graph.ids.get(s_id).unwrap().borrow().status.clone())
                                    .all(|status| match status {
                                        NodeStatus::Solved => true,
                                        _ => false,
                                    });
                                let any_failed = connector
                                    .children
                                    .iter()
                                    .map(|s_id| {
                                        self.search_graph.ids.get(s_id).unwrap().borrow().status.clone()
                                    })
                                    .any(|status| match status {
                                        NodeStatus::Failed => true,
                                        _ => false,
                                    });
                                let node_status = match (all_solved, any_failed) {
                                    (true, _) => NodeStatus::Solved,
                                    (_, true) => NodeStatus::Failed,
                                    _ => NodeStatus::OnGoing,
                                };
                                costs.push((index, connector_cost + succ_cost, node_status));
                            }
                            let (mut min_index, mut min_cost, mut node_status) =
                                (usize::MAX, f32::INFINITY, NodeStatus::OnGoing);
                            for (index, cost, status) in costs {
                                if cost < min_cost {
                                    min_index = index;
                                    min_cost = cost;
                                    node_status = status;
                                }
                            }
                            p_node.mark(min_index as u32);
                            if node_status.is_terminal() {
                                p_node.status = node_status;
                            }
                            self.open.remove(&p);
                            cost_prop_queue.push_back(*p);
                        } else {
                            let conenction_index = self.child_parent_connector_index(&connectors, m);
                            let connector = &connectors.children[conenction_index];
                            // TODO: else if p is an or node
                            if connector.children.len() == 1 {
                                // TODO: change h_val 
                                let cost = connector.cost + m_node.cost.max(0.0);
                                if cost < p_node.cost {
                                    p_node.cost = cost.clone();
                                    // TODO: if f_old box
                                    if self.prev_cost.get(&p).unwrap() < &cost {
                                        self.open.insert(*p);
                                    } else {
                                        self.open.remove(&p);
                                        p_node.mark(conenction_index as u32);
                                        match m_node.status {
                                            NodeStatus::OnGoing => {},
                                            NodeStatus::Failed => {
                                                p_node.status = NodeStatus::Failed;
                                            },
                                            NodeStatus::Solved => {
                                                p_node.status = NodeStatus::Solved;
                                            }
                                        }
                                        cost_prop_queue.push_back(*p);
                                    }
                                }
                            } else {
                                // TODO: else
                                cost_prop_queue.push_back(*p);
                            }
                        }
                    }
                }
            }
        }
    }

    fn all_children_found(&self, connectors: &NodeConnections) -> bool {
        for connector in connectors.children.iter() {
            if connector.children.iter().any(|x| !self.found.contains(x)) {
                return false;
            }
        }
        true
    }

    fn child_parent_connector_index(&self, p_connections: &NodeConnections, m: u32) -> usize {
        for (index, connector) in p_connections.children.iter().enumerate() {
            if connector.children.contains(&m) {
                return index;
            }
        }
        panic!("not a child")
    }

    // pub fn cfc_rev_star(problem: &FONDProblem, h_type: HeuristicType) -> SearchResult {
    //     // definitive cost assignment

    //     // cost prop
    //     let mut cost_prop_queue = VecDeque::from([m]);
    //     while !cost_prop_queue.is_empty() {

    //                 if z.contains(p_id) {
    //                     let mut parent_node = search_graph.ids.get(p_id).unwrap().borrow_mut();
    //                     if let Some(mut connectors) = parent_node.connections.as_mut() {
    //                         if all_children_found(&connectors, &found) {

    //                         } else {
    //                             // TODO: if p is an or node
    //                             let mut child_index;
    //                             for (index, child) in parent_node
    //                                 .connections
    //                                 .as_ref()
    //                                 .unwrap()
    //                                 .children
    //                                 .iter()
    //                                 .enumerate()
    //                             {
    //                                 if child.children.contains(&m) {
    //                                     child_index = index;
    //                                     break;
    //                                 }
    //                             }
    //                             let connection_cost = parent_node.connections.unwrap().children
    //                                 [child_index]
    //                                 .cost
    //                                 .clone();
    //                             let f = search_graph.ids.get(&m).unwrap().borrow().cost.clone();
    //                             let h = 0.0;
    //                             let max_2 = if f > h { f } else { h };
    //                         }
    //                     } else {
    //                         cost_prop_queue.push_back(*p_id)
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }
    // todo!();
}
