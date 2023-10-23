use std::{fmt::{write, format}, iter, collections::HashSet, u32};

use rand::random;

use crate::{search::{ComputeTree, ConnectionLabel}, task_network::HTN};
use super::NodeStatus;
pub struct ToDOT {

}

impl ToDOT {
    pub fn compute_tree(tree: &ComputeTree) -> String {
        let mut vertices = String::new();
        let mut edges = String::new();
        for (i, val) in tree.ids.iter() {
            let color = ToDOT::node_color(&val.borrow().label);
            vertices += &format!("\t{} [label={}, color={:?}]\n", i, i, color);
            match val.borrow().connections.as_ref() {
                Some(x) => {
                    let mut connectors = vec![];
                    for children in x.children.iter() {
                        connectors.push(
                            (children.children.clone(), children.is_marked, children.cost));
                    }
                    let and_string = ToDOT::and_node_string(tree, *i, connectors);
                    edges += &and_string;
                }
                None => {}
            }
        }
        format!("digraph {{\n\tcompound=true\n{}\n{}\n}}", vertices, edges)
    }

    pub fn htn(htn: &HTN) -> String {
        let node_ids = htn.get_nodes();
        let node_task: Vec<(String, bool)> = node_ids.iter().map(|x| {
            let task = htn.get_task(*x).unwrap();
            (task.get_name(), task.is_primitive())
        }).collect();
        let mut vertices = String::new();
        for (id, (name, is_primitive)) in node_ids.iter().zip(node_task.iter()) {
            vertices += &format!("\t{} [label=\"{}\"", id, name);
            if *is_primitive {
                vertices += ", color=green]\n";
            } else {
                vertices += "]\n"
            }
        }
        let orderings = htn.get_orderings();
        let mut edges = String::new();
        for (i, j) in orderings.iter() {
            edges += &format!("\t{}->{}\n", i, j);
        }
        format!("digraph {{\n{}\n{}\n}}", vertices, edges)
    }
    fn node_color(status: &NodeStatus) -> Color {
        match status {
            NodeStatus::Solved => Color::green,
            NodeStatus::Failed => Color::red,
            NodeStatus::OnGoing => Color::blue
        }
    }

    fn and_node_string(tree: &ComputeTree, id: u32, children: Vec<(HashSet<u32>, bool, f32)>) -> String {
        let mut string = String::new();
        for (child, is_marked, cost) in children.iter() {
            if child.len() == 1 {
                string += &format!("\t{}->{}", id, child.iter().next().unwrap());
                if *is_marked {
                    string += &format!("[label=\"{}\"]\n", cost);
                } else {
                    string += " [style=dashed]\n";
                }
            } else {
                let cluster_id = random::<u16>().to_string();
                string += &format!("\tsubgraph cluster{} {{\n", cluster_id);
                for node in child.iter() {
                    string+= &format!("\t\t{}\n", node);
                }
                let random_child = child.iter().next().unwrap();
                string+= &format!("\t}}\n\t{}->{} [lhead=cluster{}]", id, random_child, cluster_id);
                if *is_marked {
                    string += &format!(" [label={}]\n", cost);
                } else {
                    string += " [style=dashed]\n";
                }
            }
        }
        string
    }
}

#[derive(Debug)]
enum Color {
    red,
    green,
    blue
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::collections::{HashMap, HashSet};
    use crate::task_network::*;
    use super::*;
    use crate::search::*;
    use std::cell::RefCell;
    fn generate_tree() -> ComputeTree {
        let dummy_search_node = SearchNode {
            state: Rc::new(HashSet::new()),
            tn: Rc::new(HTN::new(
                HashSet::new(), vec![], HashMap::new()
            ))
        };
        let dummy_action = Rc::new(Task::Primitive(PrimitiveAction::new(
            "p1".to_string(), 
            1, 
            HashSet::new(),
            vec![HashSet::new(), HashSet::from([1,2])], 
            vec![HashSet::new(), HashSet::from([3])]
        )));
        let n1 = ComputeTreeNode {
            parent_id: None,
            search_node: dummy_search_node.clone(),
            connections: Some(NodeConnections { children: vec![
                HyperArc { children: HashSet::from([2]), cost: 1.0, is_marked: false, action_type: ConnectionLabel::Execution(1)},
                HyperArc { children: HashSet::from([3, 4]), cost: 1.0, is_marked: true, action_type: ConnectionLabel::Execution(2)},
                HyperArc { children: HashSet::from([5]), cost: 0.0, is_marked: false, action_type: ConnectionLabel::Decomposition},
            ]}),
            cost: 2.0,
            label: NodeStatus::OnGoing
        };
        let n2 = ComputeTreeNode {
            parent_id: Some(1),
            search_node: dummy_search_node.clone(),
            connections: None,
            cost: f32::INFINITY,
            label: NodeStatus::Failed
        };
        let n3 = ComputeTreeNode {
            parent_id: Some(1),
            search_node: dummy_search_node.clone(),
            connections: Some(NodeConnections { children: vec![
                HyperArc { children: HashSet::from([6]), cost: 1.0, is_marked: true, action_type: ConnectionLabel::Decomposition}
            ]}),
            cost: 2.0,
            label: NodeStatus::OnGoing
        };
        let n4 = ComputeTreeNode {
            parent_id: Some(1),
            search_node: dummy_search_node.clone(),
            connections: None,
            cost: 0.0,
            label: NodeStatus::Solved
        };
        let n5 = ComputeTreeNode {
            parent_id: Some(1),
            search_node: dummy_search_node.clone(),
            connections: Some(NodeConnections { children: vec![
                HyperArc { children: HashSet::from([7, 8]), cost: 1.0, is_marked: false, action_type: ConnectionLabel::Execution(3)},
            ]}),
            cost: 3.0,
            label: NodeStatus::OnGoing
        };
        let n6 = ComputeTreeNode {
            parent_id: Some(3),
            search_node: SearchNode::new(
                Rc::new(HashSet::new()),
                Rc::new(HTN::new(
                    HashSet::from([1]), 
                    vec![],
                    HashMap::from([
                        (1, dummy_action.clone())
                    ])
                ))
            ),
            connections: None,
            cost: 1.0,  
            label: NodeStatus::OnGoing
        };
        let n7 = ComputeTreeNode {
            parent_id: Some(5),
            search_node: dummy_search_node.clone(),
            connections: None,
            cost: 2.0,
            label: NodeStatus::OnGoing
        };
        let n8 = ComputeTreeNode {
            parent_id: Some(5),
            search_node: dummy_search_node.clone(),
            connections: None,
            cost: 1.0,
            label: NodeStatus::OnGoing
        };
        ComputeTree {
            ids: HashMap::from([
                (1, RefCell::new(n1)), (2, RefCell::new(n2)), (3, RefCell::new(n3)), (4, RefCell::new(n4)),
                (5, RefCell::new(n5)), (6, RefCell::new(n6)), (7, RefCell::new(n7)), (8, RefCell::new(n8))
            ]),
            root: 1,
            cursor: 9,
        }
    }
    #[test]
    pub fn compute_tree_test() {
        let tree = generate_tree();
        let dot_rep = ToDOT{};
        let string = ToDOT::compute_tree(&tree);
        println!("{}", string);
        assert_eq!(true, true);
    }
}