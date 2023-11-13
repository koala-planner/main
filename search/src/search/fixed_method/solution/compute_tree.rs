use crate::domain_description::ClassicalDomain;
use crate::relaxation::OutcomeDeterminizer;
use std::collections::{HashMap, HashSet, LinkedList, BTreeSet};
use std::vec;

use super::fm_policy::FMPolicy;
use super::ConnectionLabel;
use super::{connectors::NodeConnections, ComputeTreeNode, FONDProblem, SearchNode, SearchResult};
use super::{HyperArc, NodeStatus, HTN};
use crate::relaxation::ToClassical;
use std::cell::RefCell;
use std::rc::Rc;

// use crate::heuristic_calculator::FF;

#[derive(Debug)]
pub struct ComputeTree {
    pub ids: HashMap<u32, RefCell<ComputeTreeNode>>,
    pub root: u32,
    // Keeps teack of maximum u32 ID used in the tree
    pub cursor: u32,
    relaxed_domain: Option<(ToClassical, HashMap<u32, u32>)>,
}

impl ComputeTree  {
    pub fn new(problem: &FONDProblem) -> ComputeTree {
        let initial_tn = problem.init_tn.clone();
        let search_node =
            SearchNode::new(Rc::new(problem.initial_state.clone()), Rc::new(initial_tn));
        let compute_node = ComputeTreeNode {
            parent_id: None,
            search_node,
            connections: None,
            cost: 0.0,
            status: NodeStatus::OnGoing,
        };
        let (outcome_det, bijection) = OutcomeDeterminizer::from_fond_problem(&problem);
        let relaxed = ToClassical::new(&outcome_det);
        ComputeTree {
            ids: HashMap::from([(1, RefCell::new(compute_node))]),
            root: 1,
            cursor: 2,
            relaxed_domain: Some((relaxed, bijection)),
        }
    }

    pub fn is_terminated(&self) -> bool {
        let root = self.ids.get(&self.root).unwrap().borrow();
        match root.status {
            NodeStatus::Solved => true,
            NodeStatus::Failed => true,
            NodeStatus::OnGoing => false,
        }
    }

    pub fn get_max_cost_node(&self, nodes: &BTreeSet<u32>) -> u32 {
        let (mut argmax, mut max_cost) = (u32::MAX, f32::INFINITY);
        for id in nodes.iter() {
            let cost = self.ids.get(id).unwrap().borrow().cost;
            if cost < max_cost {
                max_cost = cost;
                argmax = *id;
            }
        }
        if argmax == u32::MAX {
            panic!("undefined behavior");
        }
        argmax
    }

    pub fn search_result(&self) -> SearchResult {
        let root = self.ids.get(&self.root).unwrap().borrow();
        match root.status {
            NodeStatus::Solved => SearchResult::Success(FMPolicy::new(self)),
            NodeStatus::Failed => SearchResult::NoSolution,
            NodeStatus::OnGoing => panic!("computation not terminated"),
        }
    }

    pub fn get_tip_nodes(&self) -> BTreeSet<u32> {
        let mut working_set = BTreeSet::from([self.root]);
        let mut tip_node_ids = BTreeSet::new();
        while !working_set.is_empty() {
            let mut marked = BTreeSet::new();
            // Follow Markers
            for x in working_set.iter() {
                let mut marker_update = None;
                {
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
                                    let (_, i) = self.compute_min_cost(connection);
                                    marked.extend(connection.children[i as usize].children.iter());
                                                                        marker_update = Some(i);
                                }
                            }
                        }
                        None => match node.status {
                            NodeStatus::OnGoing => {
                                tip_node_ids.insert(*x);
                            }
                            _ => {}
                        },
                    }
                }
                if marker_update.is_some() {
                    let mut node = self.ids.get(x).unwrap().borrow_mut();
                    node.mark(marker_update.unwrap());
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
            node.status = NodeStatus::Solved;
            node.cost = 0.0;
        } else {
            node.status = NodeStatus::Failed;
            node.cost = f32::INFINITY;
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
                (x.items.into_iter().map(|y| {
                    let mut h = 0.0;
                    if self.relaxed_domain.is_some() {
                        h = y.compute_heuristic_value(
                            &self.relaxed_domain.as_ref().unwrap().0,
                            &self.relaxed_domain.as_ref().unwrap().1
                            );
                    }
                    let mut child_label = NodeStatus::OnGoing;
                    if h == f32::INFINITY {
                        child_label = NodeStatus::Failed;
                    } else if y.is_goal(){
                        child_label = NodeStatus::Solved;
                    }
                    ComputeTreeNode {
                        parent_id: Some(id),
                        search_node: y,
                        connections: None,
                        cost: h,
                        status: child_label,
                    }
                }).collect::<Vec<ComputeTreeNode>>(), x.connection_label)
            }).collect();
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
                ConnectionLabel::Decomposition(name) => {}
                ConnectionLabel::Execution(name, cost) => connection_cost += *cost as f32,
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
    fn revise_node_cost(&mut self, id: &u32) -> Option<u32> {
        let mut node = self.ids.get(id).unwrap().borrow_mut();
        // Check whether Node is terminal or not
        match node.status {
            NodeStatus::Failed => {
                node.cost = f32::INFINITY;
                return node.parent_id;
            }
            NodeStatus::Solved => {
                node.cost = 0.0;
                return node.parent_id;
            }
            // If node is not terminal, check whether children terminated or not
            NodeStatus::OnGoing => {
                match self.children_status(node.connections.as_ref().unwrap()) {
                    NodeStatus::Failed => {
                        node.status = NodeStatus::Failed;
                        node.cost = f32::INFINITY;
                        node.clear_marks();
                        return node.parent_id;
                    }
                    NodeStatus::Solved => {
                        node.status = NodeStatus::Solved;
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
                return self.children_status(connection) 
            }
            None => return node.status.clone(),
        }
    }

    fn arc_status(&self, arc: &HyperArc) -> NodeStatus {
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
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{task_network::{Task, PrimitiveAction}, visualization::ToDOT, domain_description::DomainTasks};

    use super::*;
    fn generate_tree() -> ComputeTree {
        let dummy_action = Task::Primitive(PrimitiveAction::new(
            "dummy_action".to_string(), 
            1, 
            HashSet::new(),
            vec![HashSet::new(), HashSet::from([1,2])], 
            vec![HashSet::new(), HashSet::from([3])]
        ));
        let dummy_domain = Rc::new(DomainTasks::new(vec![dummy_action]));
        let dummy_search_node = SearchNode {
            state: Rc::new(HashSet::new()),
            tn: Rc::new(HTN::new(
                BTreeSet::new(), vec![], dummy_domain.clone(), HashMap::new()
            ))
        };
        let n1 = ComputeTreeNode {
            parent_id: None,
            search_node: dummy_search_node.clone(),
            connections: Some(NodeConnections { children: vec![
                HyperArc { children: HashSet::from([2]), cost: 1.0, is_marked: false,
                    action_type: ConnectionLabel::Execution("p1".to_string(), 1)},
                HyperArc { children: HashSet::from([3, 4]), cost: 1.0, is_marked: true,
                    action_type: ConnectionLabel::Execution("p2".to_string(), 2)},
                HyperArc { children: HashSet::from([5]), cost: 0.0, is_marked: false,
                    action_type: ConnectionLabel::Decomposition("m1".to_string())},
            ]}),
            cost: 2.0,
            status: NodeStatus::OnGoing
        };
        let n2 = ComputeTreeNode {
            parent_id: Some(1),
            search_node: dummy_search_node.clone(),
            connections: None,
            cost: f32::INFINITY,
            status: NodeStatus::Failed
        };
        let n3 = ComputeTreeNode {
            parent_id: Some(1),
            search_node: dummy_search_node.clone(),
            connections: Some(NodeConnections { children: vec![
                HyperArc { children: HashSet::from([6]), cost: 1.0, is_marked: true,
                    action_type: ConnectionLabel::Decomposition("m3".to_string())}
            ]}),
            cost: 2.0,
            status: NodeStatus::OnGoing
        };
        let n4 = ComputeTreeNode {
            parent_id: Some(1),
            search_node: dummy_search_node.clone(),
            connections: None,
            cost: 0.0,
            status: NodeStatus::Solved
        };
        let n5 = ComputeTreeNode {
            parent_id: Some(1),
            search_node: dummy_search_node.clone(),
            connections: Some(NodeConnections { children: vec![
                HyperArc { children: HashSet::from([7, 8]), cost: 1.0, is_marked: false,
                    action_type: ConnectionLabel::Execution("p3".to_string(), 1)},
            ]}),
            cost: 3.0,
            status: NodeStatus::OnGoing
        };
        let n6 = ComputeTreeNode {
            parent_id: Some(3),
            search_node: SearchNode::new(
                Rc::new(HashSet::new()),
                Rc::new(HTN::new(
                    BTreeSet::from([1]), 
                    vec![],
                    dummy_domain.clone(),
                    HashMap::from([(1, dummy_domain.get_id("dummy_action"))])
                ))
            ),
            connections: None,
            cost: 1.0,  
            status: NodeStatus::OnGoing
        };
        let n7 = ComputeTreeNode {
            parent_id: Some(5),
            search_node: dummy_search_node.clone(),
            connections: None,
            cost: 2.0,
            status: NodeStatus::OnGoing
        };
        let n8 = ComputeTreeNode {
            parent_id: Some(5),
            search_node: dummy_search_node.clone(),
            connections: None,
            cost: 1.0,
            status: NodeStatus::OnGoing
        };
        ComputeTree {
            ids: HashMap::from([
                (1, RefCell::new(n1)), (2, RefCell::new(n2)), (3, RefCell::new(n3)), (4, RefCell::new(n4)),
                (5, RefCell::new(n5)), (6, RefCell::new(n6)), (7, RefCell::new(n7)), (8, RefCell::new(n8))
            ]),
            root: 1,
            cursor: 9,
            relaxed_domain: None
        }
    }

    #[test]
    pub fn tip_nodes_test() {
        let tree = generate_tree();
        tree.ids.get(&4).unwrap().borrow_mut().status = NodeStatus::OnGoing;
        let tip_nodes = tree.get_tip_nodes();
        assert_eq!(tip_nodes.len(), 2);
        assert_eq!(tip_nodes.contains(&4), true);
        assert_eq!(tip_nodes.contains(&6), true);
    }

    #[test]
    pub fn expansion_test() {
        let mut tree = generate_tree();
        tree.expand(6);
        assert_eq!(tree.ids.contains_key(&9), true);
        assert_eq!(tree.ids.contains_key(&10), true);
        assert_eq!(tree.ids.len(), 10);
        let n = tree.ids.get(&6).unwrap().borrow();
        match &n.connections {
            Some(x) => {
                assert_eq!(x.children.len(), 1);
                let children = &x.children[0].children;
                assert_eq!(children.contains(&9), true);
                assert_eq!(children.contains(&10), true);
            },
            None => {panic!("children not found")},
        }
        let n_child1 = tree.ids.get(&9).unwrap().borrow();
        match n_child1.parent_id {
            Some(x) => assert_eq!(x, 6),
            None => panic!("parent not found")
        }
        let n_child2 = tree.ids.get(&10).unwrap().borrow();
        match n_child2.parent_id {
            Some(x) => assert_eq!(x, 6),
            None => panic!("parent not found")
        }
    }

    // #[test]
    // pub fn node_failure_revise_test() {
    //     let mut tree = generate_tree();
    //     let action = Task::Primitive(PrimitiveAction {
    //         name: "p".to_string(),
    //         cost: 1, pre_cond: HashSet::from([1,2]), add_effects: vec![], del_effects: vec![]
    //     });
    //     {
    //         let mut node = tree.ids.get(&6).unwrap().borrow_mut();
    //         node.search_node = SearchNode {
    //             state: Rc::new(HashSet::new()),
    //             tn: Rc::new(HTN::new(
    //                 BTreeSet::from([1]),
    //                 vec![],
    //                 Rc::new(DomainTasks::new(vec![])),
    //                 HashMap::from([(1, Rc::new(action))])
    //             ))
    //         }
    //     }
    //     tree.expand(6);
    //     assert_eq!(tree.ids.len(), 8);
    //     tree.backward_cost_revision(6);
    //     let failed_node = tree.ids.get(&6).unwrap().borrow();
    //     match failed_node.status {
    //         NodeStatus::Failed => {},
    //         _ => {panic!("node label is incorrect")}
    //     }
    //     assert_eq!(failed_node.cost, f32::INFINITY);
    //     let parent_node = tree.ids.get(&3).unwrap().borrow();
    //     match parent_node.status {
    //         NodeStatus::Failed => {},
    //         _ => {panic!("node label is incorrect")}
    //     }
    //     assert_eq!(parent_node.get_marked_connection().is_none(), true);
    //     let root = tree.ids.get(&1).unwrap().borrow();
    //     match root.status {
    //         NodeStatus::OnGoing => {},
    //         _ => {panic!("root label is incorrect")}
    //     }
    //     match root.get_marked_connection() {
    //         Some(x) => {
    //             assert_eq!(x.children.len(), 1);
    //             assert_eq!(x.children.contains(&5), true)
    //         },
    //         None => panic!("nodes are not marked")
    //     }
    //     let new_tip_nodes = tree.get_tip_nodes();
    //     assert_eq!(new_tip_nodes.len(), 2);
    //     assert_eq!(new_tip_nodes.contains(&7), true);
    //     assert_eq!(new_tip_nodes.contains(&8), true);
    //     let node = tree.ids.get(&5).unwrap().borrow();
    //     match node.connections.as_ref().unwrap().has_marked_connection() {
    //         Some(x)=> {
    //             assert_eq!(x.children.len(), 2);
    //             assert_eq!(x.children.contains(&7), true);
    //             assert_eq!(x.children.contains(&8), true);
    //         },
    //         None => panic!("wrong markers")
    //     }
    // }
}
