use h_type::HeuristicType;

use crate::domain_description::{ClassicalDomain, Facts};
use crate::relaxation::OutcomeDeterminizer;
use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet, LinkedList, BTreeSet};
use std::vec;

use super::*;
use crate::relaxation::ToClassical;
use crate::domain_description::FONDProblem;
use std::cell::RefCell;
use std::rc::Rc;

// TODO: convert ids to a regular vector/array
#[derive(Debug)]
pub struct SearchGraph {
    pub ids: HashMap<u32, RefCell<SearchGraphNode>>,
    pub root: u32,
    // Keeps teack of maximum u32 ID used in the tree
    pub cursor: u32,
    pub relaxed_domain: Option<(ToClassical, HashMap<u32, u32>)>,
}

impl SearchGraph  {
    pub fn new(problem: &FONDProblem) -> SearchGraph {
        let initial_tn = problem.init_tn.clone();
        let search_node =
            SearchNode::new(Rc::new(problem.initial_state.clone()), Rc::new(initial_tn));
        // relaxed domain
        let (outcome_det, bijection) = OutcomeDeterminizer::from_fond_problem(&problem);
        let relaxed = ToClassical::new(&outcome_det);
        // initial node
        let compute_node = SearchGraphNode {
            parents: None,
            search_node,
            connections: None,
            cost: 0.0,
            status: NodeStatus::OnGoing,
            depth: 0,
        };
        // search graph
        SearchGraph {
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

    pub fn search_result(&self, facts: &Facts) -> SearchResult {
        let root = self.ids.get(&self.root).unwrap().borrow();
        match root.status {
            NodeStatus::Solved => SearchResult::Success(StrongPolicy::new(facts, self)),
            NodeStatus::Failed => SearchResult::NoSolution,
            NodeStatus::OnGoing => panic!("computation not terminated"),
        }
    }


    pub fn mark_as_terminal(&mut self, id: u32) {
        let mut node = self.ids.get(&id).unwrap().borrow_mut();
        if node.search_node.is_goal() {
            node.status = NodeStatus::Solved;
            node.cost = 0.0;
        } else {
            node.status = NodeStatus::Failed;
            node.cost = f32::INFINITY;
        }
    }

    pub fn visited(&self, search_node: &SearchNode) -> Option<u32> {
        for (id, node) in self.ids.iter() {
            let node = node.borrow();
            if node.search_node == *search_node {
                return Some(*id);
            }
        }
        None
    }

    pub fn is_terminal(&self, id: &u32) -> bool {
        self.ids.get(id).unwrap().borrow().is_terminal()
    }

}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{task_network::{Task, PrimitiveAction, CompoundTask}, domain_description::DomainTasks};

    use super::*;
    fn generate_tree() -> SearchGraph {
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
        let n1 = SearchGraphNode {
            parents: None,
            search_node: dummy_search_node.clone(),
            connections: Some(NodeConnections { children: vec![
                HyperArc { children: HashSet::from([2]), cost: 1.0, is_marked: false,
                    action_type: ConnectionLabel::Execution("p1".to_string(), 1)},
                HyperArc { children: HashSet::from([3, 4]), cost: 1.0, is_marked: true,
                    action_type: ConnectionLabel::Execution("p2".to_string(), 2)},
                HyperArc { children: HashSet::from([5]), cost: 0.0, is_marked: false,
                    action_type: ConnectionLabel::Decomposition("t1".to_string(), "m1".to_string())},
            ]}),
            cost: 2.0,
            status: NodeStatus::OnGoing,
            depth: 0
        };
        let n2 = SearchGraphNode {
            parents: Some(vec![1]),
            search_node: dummy_search_node.clone(),
            connections: None,
            cost: f32::INFINITY,
            status: NodeStatus::Failed,
            depth: 1
        };
        let n3 = SearchGraphNode {
            parents: Some(vec![1]),
            search_node: dummy_search_node.clone(),
            connections: Some(NodeConnections { children: vec![
                HyperArc { children: HashSet::from([6]), cost: 1.0, is_marked: true,
                    action_type: ConnectionLabel::Decomposition("t1".to_string(), "m3".to_string())}
            ]}),
            cost: 2.0,
            status: NodeStatus::OnGoing,
            depth: 1
        };
        let n4 = SearchGraphNode {
            parents: Some(vec![1]),
            search_node: dummy_search_node.clone(),
            connections: None,
            cost: 0.0,
            status: NodeStatus::Solved,
            depth: 1
        };
        let n5 = SearchGraphNode {
            parents: Some(vec![1]),
            search_node: dummy_search_node.clone(),
            connections: Some(NodeConnections { children: vec![
                HyperArc { children: HashSet::from([7, 8]), cost: 1.0, is_marked: false,
                    action_type: ConnectionLabel::Execution("p3".to_string(), 1)},
            ]}),
            cost: 3.0,
            status: NodeStatus::OnGoing,
            depth: 1
        };
        let n6 = SearchGraphNode {
            parents: Some(vec![3]),
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
            status: NodeStatus::OnGoing,
            depth: 2
        };
        let n7 = SearchGraphNode {
            parents: Some(vec![5]),
            search_node: dummy_search_node.clone(),
            connections: None,
            cost: 2.0,
            status: NodeStatus::OnGoing,
            depth: 2
        };
        let n8 = SearchGraphNode {
            parents: Some(vec![5]),
            search_node: dummy_search_node.clone(),
            connections: None,
            cost: 1.0,
            status: NodeStatus::OnGoing,
            depth: 2
        };
        SearchGraph {
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
        let tip_node = tree.find_a_tip_node();
        assert_eq!([4, 6].contains(&tip_node), true);
    }

    // TODO: Sometimes panic by attempting to add parent to root
    #[test]
    pub fn expansion_test() {
        let mut tree = generate_tree();
        tree.expand(6, &HeuristicType::HFF);
        assert_eq!(tree.ids.contains_key(&9), true);
        assert_eq!(tree.ids.len(), 9);
        let n = tree.ids.get(&6).unwrap().borrow();
        match &n.connections {
            Some(x) => {
                assert_eq!(x.children.len(), 1);
                let children = &x.children[0].children;
                assert_eq!(children.contains(&9), true);
            },
            None => {panic!("children not found")},
        }
        let n_child1 = tree.ids.get(&9).unwrap().borrow();
        match &n_child1.parents {
            Some(x) => assert_eq!(*x, vec![6]),
            None => panic!("parent not found")
        }
    }

    #[test]
    pub fn cycle_detection_test() {
        let t1 = Task::Compound(CompoundTask::new("t1".to_string(), vec![]));
        let t2 = Task::Compound(CompoundTask::new("t2".to_string(), vec![]));
        let domain = Rc::new(DomainTasks::new(vec![t1, t2]));
        let n1 = SearchGraphNode {
            parents: Some(vec![1]),
            search_node: SearchNode {
                state: Rc::new(HashSet::from([1,2])),
                tn: Rc::new(
                    HTN::new(
                        BTreeSet::from([1,2]), 
                        vec![(1,2)], 
                        domain.clone(),
                HashMap::from([(1,0), (2,1)])
                    )
                )
            },
            connections: None,
            cost: 10.0,
            status: NodeStatus::OnGoing,
            depth: 0
        };
        let n2 = SearchNode {
            state: Rc::new(HashSet::from([1,2])),
            tn: Rc::new(
                HTN::new(
                    BTreeSet::from([4,5]), 
                    vec![(4,5)], 
                    domain.clone(),
            HashMap::from([(4,0), (5,1)])
                )
            )
        };
        let graph = SearchGraph {
            ids: HashMap::from([(1, RefCell::new(n1))]),
            root: 1,
            cursor: 2,
            relaxed_domain: None
        };
        let visited = graph.visited(&n2);
        assert_eq!(true, visited.is_some());
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
