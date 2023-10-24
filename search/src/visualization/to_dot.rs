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
            let color = ToDOT::node_color(&val.borrow().status);
            vertices += &format!("\t{} [label={}, color={:?}]\n", i, i, color);
            match val.borrow().connections.as_ref() {
                Some(x) => {
                    let mut connectors = vec![];
                    for children in x.children.iter() {
                        connectors.push(
                            (children.children.clone(), children.is_marked, children.cost, children.action_type.get_label())
                        );
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

    fn and_node_string(tree: &ComputeTree, id: u32, children: Vec<(HashSet<u32>, bool, f32, String)>) -> String {
        let mut string = String::new();
        for (child, is_marked, cost, label) in children.iter() {
            if child.len() == 1 {
                string += &format!("\t{}->{}", id, child.iter().next().unwrap());
                if *is_marked {
                    string += &format!("[label=\"{}\"]\n", label);
                } else {
                    string += &format!(" [label=\"{}\",style=dashed]\n", label);
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
                    string += &format!(" [label={}]\n", label);
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
