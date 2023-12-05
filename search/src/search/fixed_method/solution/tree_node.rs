use super::{SearchNode, NodeConnections, connectors::HyperArc};

#[derive(Debug)]
pub struct ComputeTreeNode {
    pub parent_id: Option<u32>,
    pub search_node: SearchNode,
    pub connections: Option<NodeConnections>,
    pub cost: f32,
    pub status: NodeStatus,
}

#[derive(Debug, Clone)]
pub enum NodeStatus {
    Solved,
    OnGoing,
    Failed
}

impl NodeStatus {
    pub fn is_terminal(&self) -> bool {
        match self {
            Self::Failed => true,
            Self::Solved => true,
            Self::OnGoing => false
        }
    }
}

impl ComputeTreeNode {
    pub fn mark(&mut self, i: u32) {
        self.clear_marks();
        self.connections.as_mut().unwrap().mark(i)
    }
    pub fn get_marked_connection(&self) -> Option<&HyperArc> {
        for item in self.connections.as_ref().unwrap().children.iter() {
            if item.is_marked {
                return Some(item);
            }
        }
        None
    }
    pub fn clear_marks(&mut self) {
        self.connections.as_mut().unwrap().clear_marks()
    }

    pub fn has_children(&self) -> bool {
        match self.connections {
            Some(_) => true,
            None => false
        }
    }

    pub fn fail(&mut self) {
        self.cost = f32::INFINITY;
        self.status = NodeStatus::Failed;
    } 

    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }
}