use super::*;

use std::collections::{HashMap,BTreeSet, HashSet};


pub fn vf2_isomorphism(g1: &Graph, g2: &Graph, l1: &HashMap<u32, u32>, l2: &HashMap<u32, u32>) -> bool {
    let mut fringe: Vec<Vec<(u32,u32)>> = Vec::new();
    // assert total isomorphism
    if l1.len() != l2.len() {
        return false
    }
    fringe.push(vec![]);
    while !fringe.is_empty() {
        let state = fringe.pop().unwrap();
        if state.len() == g1.nodes.len() {
            return true;
        }
        // Compute P
        let mut p: Vec<(u32, u32)> = vec![];
        // immediate predecessors of state
        let in_1: HashSet<u32> = state.iter()
            .map(|(n1, _)| g1.get_incoming_edges(*n1)).flatten().collect();
        let in_2: HashSet<u32> = state.iter()
            .map(|(_, n2)| g2.get_incoming_edges(*n2)).flatten().collect();
        // immediate predecessors of state
        let out_1: HashSet<u32> = state.iter()
            .map(|(n1, _)| g1.get_outgoing_edges(*n1)).flatten().collect();
        let out_2: HashSet<u32> = state.iter()
            .map(|(_, n2)| g2.get_outgoing_edges(*n2)).flatten().collect();
        // rule based construction
        // // if both "out"s are non-empty
        if (out_1.len() > 0) && (out_2.len() > 0) {
            for p1 in out_1.iter() {
                for p2 in out_2.iter() {
                    if l1.get(&p1).unwrap() == l2.get(&p2).unwrap() {
                        p.push((*p1, *p2));
                    }
                }
            }
        // // if both "in"s are non-empty
        } else if (in_1.len() > 0) && (in_2.len() > 0) {
            for p1 in in_1.iter() {
                for p2 in in_2.iter() {
                    if l1.get(&p1).unwrap() == l2.get(&p2).unwrap() {
                        p.push((*p1, *p2));
                    }
                }
            }
        // // if both are empty
        } else {
            let p1_list: Vec<u32> = g1.nodes.difference(
                &BTreeSet::from_iter(state.iter().map(|(x, _)| *x))
            ).cloned().collect();
            let p2_list: Vec<u32> = g2.nodes.difference(
                &BTreeSet::from_iter(state.iter().map(|(_, x)| *x))
            ).cloned().collect();
            for p1 in p1_list.iter() {
                for p2 in p2_list.iter() {
                    if l1.get(p1).unwrap() == l2.get(p2).unwrap() {
                        p.push((*p1, *p2));
                    }
                }
            }
        }
        for (n, m) in p.iter() {
            if state.contains(&(*n, *m)) {
                continue;
            }
            // assert same predecesssors labels
            let pred_n_list: BTreeSet<u32> = g1.get_incoming_edges(*n)
                .iter().map(|x| *l1.get(x).unwrap()).collect();
            let pred_m_list: BTreeSet<u32> = g2.get_incoming_edges(*m)
                .iter().map(|x| *l2.get(x).unwrap()).collect();
            if pred_n_list != pred_m_list {
                continue;
            }
            // assert same successors
            let succ_n_list: BTreeSet<u32> = g1.get_outgoing_edges(*n)
                .iter().map(|x| *l1.get(x).unwrap()).collect();
            let succ_m_list: BTreeSet<u32> = g2.get_outgoing_edges(*m)
                .iter().map(|x| *l2.get(x).unwrap()).collect();
            if succ_n_list != succ_m_list {
                continue;
            }
            let mut new_state = state.clone();
            new_state.push((*n,*m));
            fringe.push(new_state);
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_test_instances() -> (Graph, Graph, Graph, Graph) {
        let n1 = BTreeSet::from([1,2,3,4]);
        let n2 = BTreeSet::from([4,5,6,7]);
        let n3 = BTreeSet::from([1,2,3]);
        let n4 = BTreeSet::from([4,5,6,7]);
        let g1 = Graph::new(n1, vec![(1,3), (2,3), (3,4)]);
        let g2 = Graph::new(n2, vec![(4,6), (5,6), (6,7)]);
        let g3 = Graph::new(n3, vec![(1,2), (1,3)]);
        let g4 = Graph::new(n4, vec![(4,5), (5,6), (6,7)]);
        (g1, g2, g3, g4)
    }

    #[test]
    pub fn vf2_correctness_test() {
        let (g1, g2, g3, g4) = generate_test_instances();
        let l1 = HashMap::from([(1,1), (2,2), (3,3), (4,2)]);
        let l2 = HashMap::from([(4,1), (5,2), (6,3), (7,2)]);
        let result = vf2_isomorphism(&g1, &g2, &l1, &l2);
        let l2_2 = HashMap::from([(4,1), (5,2), (6,3), (7,4)]);
        let result = vf2_isomorphism(&g1, &g2, &l1, &l2_2);
        assert_eq!(result, false);
        let l3 = HashMap::from([(1,1), (2,3), (3,3)]);
        let result = vf2_isomorphism(&g1, &g3, &l1, &l3);
        assert_eq!(result, false);
        let result = vf2_isomorphism(&g3, &g1, &l3, &l1);
        assert_eq!(result, false);
        let l4 = HashMap::from([(4,1), (5,2), (6,3), (7,2)]);
        let result = vf2_isomorphism(&g2, &g4, &l2, &l4);
        assert_eq!(result, false);
    }

    #[test]
    pub fn vf2_correctness_test2() {
        let g1 = Graph::new(BTreeSet::from([5,6]), vec![(5,6)]);
        let g2 = Graph::new(BTreeSet::from([1,2]), vec![(1,2)]);
        let l1 = HashMap::from([(5,1), (6,2)]);
        let l2 = HashMap::from([(1,1), (2,2)]);
        assert_eq!(vf2_isomorphism(&g1, &g2, &l1, &l2), true);
    }
}