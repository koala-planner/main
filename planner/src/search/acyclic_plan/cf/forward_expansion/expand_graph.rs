use super::*;
use std::cell::RefCell;
use std::collections::{HashSet, BTreeSet};

impl SearchGraph {
    pub fn find_a_tip_node(&self) -> u32 {
        let mut working_set = BTreeSet::from([self.root]);
        let (mut candidate, mut depth, mut cost) = (u32::MIN, u16::MIN, f32::NEG_INFINITY);
        while !working_set.is_empty() {
            let x = working_set.pop_first().unwrap();
            let mut node = self.ids.get(&x).unwrap().borrow_mut();
            match node.status {
                NodeStatus::Solved => {continue;}
                NodeStatus::Failed => {continue;}
                NodeStatus::OnGoing => {
                    match &node.connections {
                        Some(succ) => {
                            match node.get_marked_connection() {
                                Some(marked) => {
                                    match self.arc_status(marked) {
                                        NodeStatus::OnGoing => {
                                            working_set.extend(marked.children.iter());
                                        },
                                        _ => {
                                            if node.depth >= depth {
                                                if node.cost >= cost {
                                                    candidate = x;
                                                }
                                            }
                                        }
                                    }
                                },
                                None => {
                                    if node.depth >= depth {
                                        if node.cost >= cost {
                                            candidate = x;
                                        }
                                    }
                                }
                            }
                        },
                        None => {
                            return x;
                        }
                    }
                }
            }
        }
        return candidate
    }

    pub fn expand(&mut self, id: u32, h_type: &HeuristicType) {
        // if node's successor's has already been found, skip
        if let Some(_) = self.ids.get(&id).unwrap().borrow().connections {
            return;
        }
        // compute successors
        let node_successors = self.ids.get(&id).unwrap().borrow().search_node.expand();
        let depth = self.ids.get(&id).unwrap().borrow().depth.clone();
        // Case where node is terminal, terminate expansion
        if node_successors.len() == 0 {
            self.mark_as_terminal(id);
            return;
        }
        let mut connectors = vec![];
        for expansion in node_successors.into_iter() {
            let mut hyperarc = HyperArc {
                children: HashSet::new(),
                cost: 1.0,
                is_marked: false,
                action_type: expansion.connection_label
            };
            let subproblems = expansion.items;
            for subproblem in subproblems {
                let visited_before = self.visited(&subproblem);
                match visited_before {
                    Some(x) => {
                        self.ids.get(&x).unwrap().borrow_mut().add_parent(x);
                        hyperarc.children.insert(x);
                    },
                    None => {
                        let mut h = 0.0;
                        match &self.relaxed_domain {
                            Some((encoder, bijection)) => {
                                h = subproblem.compute_heuristic_value(encoder, bijection, &h_type)
                            },
                            None => {}
                        }
                        let mut subproblem_label = NodeStatus::OnGoing;
                        if h == f32::INFINITY {
                            subproblem_label = NodeStatus::Failed;
                        } else if subproblem.is_goal() {
                            subproblem_label = NodeStatus::Solved;
                        }
                        let new_subproblem = SearchGraphNode {
                            parents: Some(vec![id]),
                            search_node: subproblem,
                            connections: None,
                            cost: h,
                            status: subproblem_label,
                            depth: depth + 1
                        };
                        self.ids.insert(self.cursor, RefCell::new(new_subproblem));
                        hyperarc.children.insert(self.cursor);
                        self.cursor += 1;
                    }
                }
            }
            connectors.push(hyperarc);
        }
        self.ids.get(&id).unwrap().borrow_mut().connections = Some(NodeConnections { children: connectors });
    }
}