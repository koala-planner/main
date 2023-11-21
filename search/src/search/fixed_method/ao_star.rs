use std::collections::{BinaryHeap, HashSet, HashMap};
use crate::{domain_description::FONDProblem, task_network::HTN, visualization::ToDOT};

use super::{solution::SearchResult, solution::ComputeTree};

pub struct AOStarSearch {

}
impl AOStarSearch {
    // the initial TN is assumed to be in collapsed format (i.e., with a single abstract task)
    pub fn run(problem: &FONDProblem) -> SearchResult {
        let mut compute_tree = ComputeTree::new(problem);
        while !compute_tree.is_terminated() {
            let tip_nodes = compute_tree.get_tip_nodes();
            let n = compute_tree.get_max_cost_node(&tip_nodes);
            compute_tree.expand(n);
            compute_tree.backward_cost_revision(n);
        }
        compute_tree.search_result(&problem.facts)
    }
}