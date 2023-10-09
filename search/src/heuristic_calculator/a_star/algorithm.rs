use std::collections::{BinaryHeap, HashSet, HashMap};
use std::rc::Rc;

use crate::{domain_description::ClassicalDomain, task_network::Applicability};
use super::{AStarResult, SearchNode};

pub struct AStar{ }

impl AStar {
    //
    pub fn solve(problem: &ClassicalDomain) -> AStarResult {
        let init_search_node = SearchNode {
            g: 0.0,
            h: AStar::compute_h(&problem.init_state, &problem.goal_state) as f32,
            state: Rc::new(problem.init_state.clone()),
            hist: vec![]
        };
        let mut visited: Vec<(Rc<HashSet<u32>>, f32)> = vec![];
        let mut fringe: BinaryHeap<SearchNode> = BinaryHeap::from([init_search_node]);
        while !fringe.is_empty() {
            let node = fringe.pop().unwrap();
            if node.h == 0.0 {
                return AStarResult::Success(node.g);
            }
            for action in problem.actions.iter() {
                if action.is_applicable(&node.state) {
                    // A* can only solve deterministic action effects
                    let new_state = action.transition(&node.state)[0].to_owned();
                    let pos = visited.iter().position(|(x, i)| x.as_ref() == &new_state);
                    let g = node.g + action.cost as f32;
                    let mut has_been_visited = false;
                    match pos {
                        Some(i) => {
                            let val = visited[i].1;
                            if g >= val {
                                has_been_visited = true;
                                continue;
                            } else {
                                visited.remove(i);
                            }
                        },
                        None => { }
                    }
                    if has_been_visited {
                        continue;
                    }
                    let h = AStar::compute_h(&new_state, &problem.goal_state);
                    let new_state = Rc::new(new_state);
                    let mut new_hist = node.hist.clone();
                    new_hist.push(action.name.clone());
                    let new_node = SearchNode {
                        g: g,
                        h: h,
                        state: new_state.clone(),
                        hist: new_hist
                    };
                    fringe.push(new_node);
                    visited.push((new_state, g));
                }
            }
        }
        AStarResult::NoSolution
    }

    pub fn compute_h(state: &HashSet<u32>, goal: &HashSet<u32>) -> f32 {
        goal.difference(state).count() as f32
    }
}


#[cfg(test)]
mod test {
    use std::ops::Index;

    use super::*;
    use crate::{task_network::PrimitiveAction, domain_description::Facts};

    #[test]
    pub fn heuristic_val_test() {
        let state = HashSet::from([1,2,3,4,5]);
        let goal = HashSet::from([2,3,4,6]);
        let h = AStar::compute_h(&state, &goal);
        assert_eq!(h, 1.0);
    }

    #[test]
    pub fn correctness_test() {
        let state = HashSet::from([1]);
        let goal = HashSet::from([2,3,4,5]);
        let mut actions = vec![
            PrimitiveAction::new(
                "p1".to_string(),
                2,
                HashSet::from([1]),
                vec![HashSet::from([2,3])], 
                vec![HashSet::from([1])]
            ),
            PrimitiveAction::new(
                "p2".to_string(),
                1,
                HashSet::from([2,3]),
                vec![HashSet::from([1,4,5])], 
                vec![HashSet::from([2])]
            ),
            PrimitiveAction::new(
                "p3".to_string(),
                3,
                HashSet::from([4,5]),
                vec![HashSet::from([2]),], 
                vec![HashSet::new(),]
            ),
            PrimitiveAction::new(
                "p3_2".to_string(),
                1,
                HashSet::from([4,5]),
                vec![HashSet::from([2]),], 
                vec![HashSet::new(),]
            )
        ];
        let facts = Facts::new(vec![
            "1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string()
        ]);
        let mut domain = ClassicalDomain::new(
            facts.clone(),
            actions.clone(),
            state.clone(),
            goal.clone()
        );
        let result = AStar::solve(&domain);
        match result {
            AStarResult::Success(x) => {
                assert_eq!(x, 4.0);
            },
            _ => panic!()
        }
        let goal = HashSet::from([6]);
        let domain = ClassicalDomain::new(facts, actions, state, goal);
        let result = AStar::solve(&domain);
        match result {
            AStarResult::Success(_) => {
                panic!()
            },
            AStarResult::NoSolution => {}
        }
    }
}