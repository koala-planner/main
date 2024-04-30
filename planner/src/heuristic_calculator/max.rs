use std::collections::HashSet;
use super::*;

pub fn h_max(domain: &ClassicalDomain, state: &HashSet<u32>, goal: &HashSet<u32>) -> f32  {
    let graphplan = GraphPlan::build_graph(
        domain,
        state,
        goal
    );
    match graphplan {
        Some(graph) => (graph.depth / 2) as f32,
        None => f32::INFINITY
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::heuristic_calculator::PrimitiveAction;
    use crate::domain_description::Facts;

    pub fn generate_domain() -> ClassicalDomain {
        let p1 = PrimitiveAction::new(
            "p1".to_string(), 
            1, 
            HashSet::from([0]), 
            vec![HashSet::from([1]),],
            vec![HashSet::from([3]),] 
        );
        let p2 = PrimitiveAction::new(
            "p2".to_string(), 
            1, 
            HashSet::from([1]), 
            vec![HashSet::from([2]),],
            vec![HashSet::new(),] 
        );
        let p3 = PrimitiveAction::new(
            "p3".to_string(), 
            1, 
            HashSet::from([1]), 
            vec![HashSet::from([3]),],
            vec![HashSet::new(),] 
        );
        let p4 = PrimitiveAction::new(
            "p4".to_string(), 
            1, 
            HashSet::from([1, 2, 3]), 
            vec![HashSet::from([4]),],
            vec![HashSet::new(),] 
        );
        let facts = Facts::new(vec![
            "0".to_owned(), "1".to_owned(), "2".to_owned(), "3".to_owned(), "4".to_owned()
        ]);
        let actions = vec![p1, p2, p3, p4];
        ClassicalDomain::new(facts, actions)
    }

    #[test]
    pub fn h_val_test() {
        let domain = generate_domain();
        let h = h_max(&domain, &HashSet::from([0]), &HashSet::from([4]));
        assert_eq!(h, 3.0);
    }
}