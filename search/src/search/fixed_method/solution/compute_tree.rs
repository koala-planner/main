use crate::domain_description::ClassicalDomain;
use crate::relaxation::{DeleteRelaxation, OutcomeDeterminizer};
use core::panic;
use std::collections::{HashMap, HashSet, LinkedList};

use super::fm_policy::FMPolicy;
use super::ConnectionLabel;
use super::{connectors::NodeConnections, ComputeTreeNode, FONDProblem, SearchNode, SearchResult};
use super::{Connector, HyperArc, NodeStatus, HTN};
use crate::relaxation::ToClassical;
use std::cell::RefCell;
use std::rc::Rc;

use crate::heuristic_calculator::FF;

#[derive(Debug)]
pub struct ComputeTree {
    pub ids: HashMap<u32, RefCell<ComputeTreeNode>>,
    pub root: u32,
    // Keeps teack of maximum u32 ID used in the tree
    cursor: u32,
    relaxed_domain: ToClassical,
}

impl ComputeTree {
    pub fn new(problem: &FONDProblem) -> ComputeTree {
        let initial_tn = problem.init_tn.clone();
        let search_node =
            SearchNode::new(Rc::new(problem.initial_state.clone()), Rc::new(initial_tn));
        let compute_node = ComputeTreeNode {
            parent_id: None,
            search_node,
            connections: None,
            cost: 0.0,
            label: NodeStatus::OnGoing,
        };
        let outcome_det = OutcomeDeterminizer::htn(&problem);
        let relaxed = ToClassical::new(&outcome_det);
        ComputeTree {
            ids: HashMap::from([(1, RefCell::new(compute_node))]),
            root: 1,
            cursor: 2,
            relaxed_domain: relaxed,
        }
    }

    pub fn is_terminated(&self) -> bool {
        let root = self.ids.get(&self.root).unwrap().borrow();
        match root.label {
            NodeStatus::Solved => true,
            NodeStatus::Failed => true,
            NodeStatus::OnGoing => false,
        }
    }

    pub fn search_result(&self) -> SearchResult {
        let root = self.ids.get(&self.root).unwrap().borrow();
        match root.label {
            NodeStatus::Solved => SearchResult::Success(FMPolicy::new(self)),
            NodeStatus::Failed => SearchResult::NoSolution,
            NodeStatus::OnGoing => panic!("computation not terminated"),
        }
    }

    // TODO: test this function
    pub fn get_tip_nodes(&self) -> HashSet<u32> {
        let mut working_set = HashSet::from([self.root]);
        let mut tip_node_ids: HashSet<u32> = HashSet::new();
        while !working_set.is_empty() {
            let mut marked = HashSet::new();
            // Follow Markers
            for x in working_set.iter() {
                let node = self.ids.get(x).unwrap().borrow();
                match &node.connections {
                    Some(connection) => {
                        let node_status = self.compute_node_status(*x);
                        if node_status.is_terminal() {
                            continue;
                        }
                        match connection.has_marked_connection() {
                            Some(children) => {
                                marked.extend(children.children.iter());
                            }
                            None => {
                                panic!()
                            }
                        }
                    }
                    None => match node.label {
                        NodeStatus::OnGoing => {
                            tip_node_ids.insert(*x);
                        }
                        _ => {}
                    },
                }
            }
            if marked.is_empty() {
                return tip_node_ids;
            } else {
                working_set = marked;
            }
        }
        tip_node_ids
    }

    fn mark_as_terminal(&mut self, id: u32) {
        let mut node = self.ids.get_mut(&id).unwrap().borrow_mut();
        if node.search_node.is_goal() {
            node.label = NodeStatus::Solved;
        } else {
            node.label = NodeStatus::Failed;
        }
    }

    pub fn expand(&mut self, id: u32) {
        // compute successors
        let node_successors = self.ids.get(&id).unwrap().borrow().search_node.expand();
        // Case where node is terminal, terminate expansion
        if node_successors.len() == 0 {
            self.mark_as_terminal(id);
            return;
        }
        // Process node expansion to the desired format
        let expansions: Vec<(Vec<ComputeTreeNode>, ConnectionLabel)> = node_successors
            .into_iter()
            .map(|x| {
                (
                    x.items
                        .into_iter()
                        .map(|y| {
                            let h = y.compute_heuristic_value(&self.relaxed_domain);
                            let child_label = NodeStatus::OnGoing;
                            ComputeTreeNode {
                                parent_id: Some(id),
                                search_node: y,
                                connections: None,
                                cost: h,
                                label: NodeStatus::OnGoing,
                            }
                        })
                        .collect::<Vec<ComputeTreeNode>>(),
                    x.connection_label,
                )
            })
            .collect();
        let mut node_connections = vec![];
        for (expansion_nodes, action_type) in expansions.into_iter() {
            let mut children_id = HashSet::new();
            for node in expansion_nodes {
                self.ids.insert(self.cursor, RefCell::new(node));
                children_id.insert(self.cursor);
                self.cursor += 1;
            }
            let mut connection_cost = 0.0;
            match &action_type {
                ConnectionLabel::Decomposition => {}
                ConnectionLabel::Execution(cost) => connection_cost += *cost as f32,
            }
            node_connections.push(HyperArc {
                children: children_id,
                cost: connection_cost,
                is_marked: false,
                action_type: action_type,
            });
        }
        self.ids.get_mut(&id).unwrap().borrow_mut().connections =
            Some(NodeConnections::new(node_connections));
    }

    fn is_terminal(&self, id: &u32) -> bool {
        self.ids.get(id).unwrap().borrow().is_terminal()
    }

    // return parent ID if further revision is needed
    // TODO: test
    fn revise_node_cost(&mut self, id: &u32) -> Option<u32> {
        let mut node = self.ids.get(id).unwrap().borrow_mut();
        // Check whether Node is terminal or not
        match node.label {
            NodeStatus::Failed => {
                node.cost = f32::INFINITY;
                // TODO: update node cost
                match node.parent_id {
                    Some(p_id) => {
                        let mut parent = self.ids.get(&p_id).unwrap().borrow_mut();
                        parent.clear_marks();
                        return Some(p_id);
                    }
                    None => {
                        return None;
                    }
                }
            }
            NodeStatus::Solved => {
                node.cost = 0.0;
                return node.parent_id;
            }
            // If node is not terminal, check whether children terminated or not
            NodeStatus::OnGoing => {
                match self.children_status(node.connections.as_ref().unwrap()) {
                    NodeStatus::Failed => {
                        node.label = NodeStatus::Failed;
                        return node.parent_id;
                    }
                    NodeStatus::Solved => {
                        node.label = NodeStatus::Solved;
                        return node.parent_id;
                    }
                    // children are not terminal
                    NodeStatus::OnGoing => {
                        let (min_cost, arg_min) =
                            self.compute_min_cost(node.connections.as_ref().unwrap());
                        node.mark(arg_min);
                        // If cost has changed
                        if node.cost != min_cost {
                            node.cost = min_cost;
                            return node.parent_id;
                        } else {
                            return None;
                        }
                    }
                }
            }
        }
    }

    // TODO: Test
    fn children_status(&self, connections: &NodeConnections) -> NodeStatus {
        // Is there at least one path to continue?
        let mut active = false;
        for arc in connections.children.iter() {
            let mut branch_status = NodeStatus::Solved;
            for child in arc.children.iter() {
                let child = self.ids.get(child).unwrap().borrow();
                if child.cost.is_infinite() {
                    branch_status = NodeStatus::Failed;
                    break;
                }
                match child.label {
                    NodeStatus::Failed => {
                        branch_status = NodeStatus::Failed;
                        break;
                    }
                    // Branch is solved
                    NodeStatus::Solved => {
                        return NodeStatus::Solved;
                    }
                    NodeStatus::OnGoing => {
                        branch_status = NodeStatus::OnGoing;
                        active = true;
                    }
                }
            }
        }
        if active {
            NodeStatus::OnGoing
        } else {
            NodeStatus::Failed
        }
    }

    fn compute_min_cost(&self, connections: &NodeConnections) -> (f32, u32) {
        let (mut min_cost, mut arg_min) = (f32::INFINITY, u32::max_value());
        for (i, arc) in connections.children.iter().enumerate() {
            let mut branch_cost = arc.cost;
            for child in arc.children.iter() {
                let child = self.ids.get(child).unwrap().borrow();
                branch_cost += child.cost;
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

    // Backward induction procedure
    // Corresponds to lines 8-13 in Nilson's book
    pub fn backward_cost_revision(&mut self, id: u32) {
        let mut working_set = LinkedList::from([id]);
        while !working_set.is_empty() {
            let node_id = working_set.pop_front().unwrap();
            match self.revise_node_cost(&node_id) {
                Some(x) => {
                    working_set.push_back(x);
                }
                None => {}
            }
        }
    }

    fn compute_node_status(&self, id: u32) -> NodeStatus {
        let node = self.ids.get(&id).unwrap().borrow();
        match &node.connections {
            Some(connection) => {
                let mut is_solved = true;
                let mut has_failed = true;
                for arc in connection.children.iter() {
                    let status = self.arc_status(&arc);
                    match status {
                        NodeStatus::OnGoing => {
                            is_solved = false;
                            has_failed = false;
                        }
                        NodeStatus::Failed => {
                            is_solved = false;
                        }
                        NodeStatus::Solved => {
                            has_failed = false;
                        }
                    }
                }
                if is_solved {
                    return NodeStatus::Solved;
                }
                if has_failed {
                    return NodeStatus::Failed;
                }
                NodeStatus::OnGoing
            }
            None => return node.label.clone(),
        }
    }

    fn arc_status(&self, arc: &HyperArc) -> NodeStatus {
        let mut result = NodeStatus::Solved;
        for item in arc.children.iter() {
            let node = self.ids.get(&item).unwrap().borrow();
            match node.label {
                NodeStatus::Failed => return NodeStatus::Failed,
                NodeStatus::OnGoing => result = NodeStatus::OnGoing,
                NodeStatus::Solved => {}
            }
        }
        result
    }
}
