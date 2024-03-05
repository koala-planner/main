use std::collections::{BinaryHeap, HashSet, HashMap};
use crate::{domain_description::FONDProblem, task_network::HTN, visualization::ToDOT};

use super::{SearchResult, SearchGraph, SearchStats, h_type, HeuristicType};
use std::time::{Instant, Duration};

pub struct AOStarSearch {

}
impl AOStarSearch {
    // the initial TN is assumed to be in collapsed format (i.e., with a single abstract task)
    pub fn run(problem: &FONDProblem, h_type: HeuristicType) -> (SearchResult, SearchStats) {
        let mut explored_nodes: u32 = 0;
        let mut max_depth = 0;
        let start_time = Instant::now();
        let mut compute_tree = SearchGraph::new(problem);
        while !compute_tree.is_terminated() {
            let n = compute_tree.find_a_tip_node();
            compute_tree.expand(n, &h_type);
            compute_tree.backward_cost_revision(n);
            explored_nodes+=1;
            let depth = compute_tree.ids.get(&n).unwrap().borrow().depth;
            if depth > max_depth {
                max_depth = depth;
            }
        }
        let result = compute_tree.search_result(&problem.facts);
        let stats = SearchStats {
            max_depth: max_depth,
            search_nodes: compute_tree.ids.len() as u32,
            explored_nodes: explored_nodes,
            seach_time: start_time.elapsed()
        };
        (result, stats)
    }
}