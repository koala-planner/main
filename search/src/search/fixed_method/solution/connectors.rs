use std::collections::HashSet;

use super::NodeExpansion;
use super::ConnectionLabel;

#[derive(Debug)]
pub struct NodeConnections {
    pub children: Vec<HyperArc>
}

#[derive(Debug)]
pub struct HyperArc {
    pub children: HashSet<u32>,
    pub cost: f32,
    pub is_marked: bool,
    pub action_type: ConnectionLabel
}

impl NodeConnections {
    pub fn new(children: Vec<HyperArc>) -> NodeConnections {
        NodeConnections { children }
    }

    pub fn mark(&mut self, index: u32) {
        for (i, child) in self.children.iter_mut().enumerate() {
            if i as u32 == index {
                child.is_marked = true;
            } else {
                child.is_marked = false;
            }
        }
    }

    pub fn has_marked_connection(&self) -> Option<&HyperArc> {
        for child in self.children.iter() {
            if child.is_marked == true {
                return Some(child);
            }
        }
        None
    }

    pub fn clear_marks(&mut self) {
        for child in self.children.iter_mut() {
            child.is_marked = false;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn marking_test() {
        let arc1 = HyperArc {
            children: HashSet::from([1,2,3]),
            cost: 0.0,
            is_marked: false,
            action_type: ConnectionLabel::Decomposition("t1".to_string(), "m1".to_string())
            
        };
        let arc2 = HyperArc {
            children: HashSet::from([5,4]),
            cost: 0.0,
            is_marked: true,
            action_type: ConnectionLabel::Decomposition("t1".to_string(), "m2".to_string())
        };
        let arc3 = HyperArc {
            children: HashSet::from([7,54]),
            cost: 0.0,
            is_marked: false,
            action_type: ConnectionLabel::Execution("p1".to_string(), 1)
        };
        let mut connections = NodeConnections {
            children: vec![arc1, arc2, arc3]
        };
        connections.mark(2);
        println!("{:?}", connections);
        assert_eq!(connections.children[1].is_marked, false);
        assert_eq!(connections.children[2].is_marked, true);
    }
}