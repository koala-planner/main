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
        let node = self.ids.get(&id).unwrap().borrow();
        let node_successors = progress(node.tn.clone(), node.state.clone());
        drop(node);
        let depth = self.ids.get(&id).unwrap().borrow().depth.clone();
        // Case where node is terminal, terminate expansion
        if node_successors.len() == 0 {
            self.mark_as_terminal(id);
            return;
        }
        let mut connectors = vec![];
        for expansion in node_successors.into_iter() {
            let mut hyperarc = Connector {
                children: HashSet::new(),
                cost: 1.0,
                is_marked: false,
                action_type: expansion.connection_label
            };
            for state in expansion.states.iter() {
                let visited_before = self.visited(expansion.tn.as_ref(), state.as_ref());
                match visited_before {
                    Some(x) => {
                        self.ids.get(&x).unwrap().borrow_mut().add_parent(x);
                        hyperarc.children.insert(x);
                    },
                    None => {
                        let mut h = 0.0;
                        match &self.relaxed_domain {
                            Some((encoder, bijection)) => {
                                h = SearchGraphNode::h_val(expansion.tn.as_ref(), state.as_ref(), encoder, bijection, &h_type)
                            },
                            None => {}
                        }
                        let mut node_label = NodeStatus::OnGoing;
                        if h == f32::INFINITY {
                            node_label = NodeStatus::Failed;
                        } else if expansion.tn.is_goal() {
                            node_label = NodeStatus::Solved;
                        }
                        let new_search_node = SearchGraphNode {
                            parents: Some(vec![id]),
                            tn: expansion.tn.clone(),
                            state: state.clone(),
                            connections: None,
                            cost: h,
                            status: node_label,
                            depth: depth + 1
                        };
                        self.ids.insert(self.cursor, RefCell::new(new_search_node));
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