use super::*;
use std::collections::BTreeSet;
impl SearchGraph {
    // Backward induction procedure
    // Corresponds to lines 8-13 in Nilson's book
    pub fn backward_cost_revision(&mut self, id: u32) {
        let mut working_set = BTreeSet::from([id]);
        while !working_set.is_empty() {
            let mut depths: Vec<(u32, u16)> = working_set.iter().map(|x| {
                (*x, self.ids.get(x).unwrap().borrow().depth)
            }).collect();
            depths.sort_by(|(_, depth1), (_, depth2)| depth2.cmp(depth1));
            let (node_id, _) = depths[0];
            working_set.remove(&node_id);
            match self.revise_node_cost(&node_id) {
                Some(x) => {
                    working_set.extend(x);
                }
                None => {}
            }
        }
    }

    // return parent ID if further revision is needed
    fn revise_node_cost(&mut self, id: &u32) -> Option<Vec<u32>> {
        let mut node = self.ids.get(id).unwrap().borrow_mut();
        // Check whether Node is terminal or not
        match node.status {
            NodeStatus::Failed => {
                node.cost = f32::INFINITY;
                return node.parents.clone();
            }
            NodeStatus::Solved => {
                node.cost = 0.0;
                return node.parents.clone();
            }
            // If node is not terminal, check whether children terminated or not
            NodeStatus::OnGoing => {
                match self.children_status(node.connections.as_ref().unwrap()) {
                    NodeStatus::Failed => {
                        node.status = NodeStatus::Failed;
                        node.cost = f32::INFINITY;
                        node.clear_marks();
                        return node.parents.clone();
                    }
                    NodeStatus::Solved => {
                        node.status = NodeStatus::Solved;
                        let (min_cost, arg_min) =
                            self.compute_min_cost(node.connections.as_ref().unwrap());
                        node.mark(arg_min);
                        node.cost = min_cost;
                        return node.parents.clone();
                    }
                    // children are not terminal
                    NodeStatus::OnGoing => {
                        let (min_cost, arg_min) =
                            self.compute_min_cost(node.connections.as_ref().unwrap());
                        node.mark(arg_min);
                        // If cost has changed
                        if node.cost != min_cost {
                            node.cost = min_cost;
                            return node.parents.clone();
                        } else {
                            return None;
                        }
                    }
                }
            }
        }
    }

    fn children_status(&self, connections: &NodeConnections) -> NodeStatus {
        // Is there at least one path to continue?
        let mut all_failed = true;
        for arc in connections.children.iter() {
            match self.arc_status(arc) {
                NodeStatus::Solved => return NodeStatus::Solved,
                NodeStatus::Failed => {},
                NodeStatus::OnGoing => all_failed = false
            }
        }
        if all_failed {
            NodeStatus::Failed
        } else {
            NodeStatus::OnGoing
        }
    }


    pub fn arc_status(&self, arc: &Connector) -> NodeStatus {
        let mut result = NodeStatus::Solved;
        for item in arc.children.iter() {
            let node = self.ids.get(&item).unwrap().borrow();
            match node.status {
                NodeStatus::Failed => return NodeStatus::Failed,
                NodeStatus::OnGoing => result = NodeStatus::OnGoing,
                NodeStatus::Solved => {}
            }
        }
        result
    }

    fn compute_min_cost(&self, connections: &NodeConnections) -> (f32, u32) {
        let (mut min_cost, mut arg_min) = (f32::INFINITY, u32::max_value());
        for (i, arc) in connections.children.iter().enumerate() {
            let mut branch_cost = arc.cost;
            let mut is_solved = true;
            for child in arc.children.iter() {
                let child = self.ids.get(child).unwrap().borrow();
                branch_cost += child.cost;
                match child.status {
                    NodeStatus::Solved => {},
                    _ => is_solved = false
                }
            }
            if is_solved {
                return (branch_cost, i as u32);
            }
            if branch_cost < min_cost {
                min_cost = branch_cost;
                arg_min = i as u32;
            }
        }
        if min_cost.is_infinite() {
            panic!("empty node connection")
        }
        (min_cost, arg_min)
    }
}