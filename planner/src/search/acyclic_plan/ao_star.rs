use std::collections::{BinaryHeap, HashSet, HashMap};
use crate::{domain_description::FONDProblem, task_network::HTN};

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
        let mut search_graph = SearchGraph::new(problem);
        while !search_graph.is_terminated() {
            let n = search_graph.find_a_tip_node();
            search_graph.expand(n, &h_type);
            search_graph.backward_cost_revision(n);
            explored_nodes+=1;
            let depth = search_graph.ids.get(&n).unwrap().borrow().depth;
            if depth > max_depth {
                max_depth = depth;
            }
        }
        let result = search_graph.search_result(&problem.facts);
        let stats = SearchStats {
            max_depth: max_depth,
            search_nodes: search_graph.ids.len() as u32,
            explored_nodes: explored_nodes,
            seach_time: start_time.elapsed()
        };
        (result, stats)
    }
}