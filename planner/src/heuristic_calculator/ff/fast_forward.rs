use std::collections::{HashSet, HashMap};

use crate::domain_description::ClassicalDomain;

use super::GraphPlan;

#[derive(Debug)]
pub struct FF {}

impl FF {
    pub fn calculate_h(domain: &ClassicalDomain, state: &HashSet<u32>, goal: &HashSet<u32>) -> f32 {
        let graphplan = GraphPlan::build_graph(
            domain,
            state,
            goal
        );
        match graphplan {
            Some(graph) => {
                return FF::plan_length(domain, graph, goal) as f32
            },
            None => {
                return f32::INFINITY;
            }
        }
    }

    fn plan_length(domain: &ClassicalDomain, graphplan: GraphPlan, goal_state: &HashSet<u32>) -> u32 {
        let mut len = 0;
        let mut g = graphplan.compute_goal_indices(goal_state);
        let mut marks = HashMap::new();
        for i in 0..graphplan.depth + 1 {
            marks.insert(i, HashSet::new());
        }
        for i in (1..graphplan.depth + 1).rev() {
            if g.get(&i).is_none() {
                continue;
            }
            let open_goals: HashSet<u32> = g.get(&i).unwrap().difference(marks.get(&i).unwrap()).cloned().collect();
            for open_goal in open_goals.iter() {
                let actions = graphplan.get_action_layer(i-1);
                let mut actions = domain.get_actions_by_index(actions);
                // select only actions that produce this goal
                actions = actions.iter().filter(|x|{
                    if x.add_effects.len() > 1 {
                        panic!("actions are not determinized")
                    }
                    x.add_effects[0].contains(open_goal)
                }).map(|x| *x).collect();
                //select min cost action
                let min_action = *actions.iter().reduce(|acc, e| {
                    if acc.cost > e.cost {
                        e
                    } else {
                        acc
                    }
                }).unwrap();
                len += 1;
                // add preconds as new goals
                // // not satisifed at the initial state
                let mut open_preconds: HashSet<u32> = min_action.pre_cond.difference(&graphplan.get_fact_layer(0)).cloned().collect();
                // // not satisfied by action
                open_preconds = open_preconds.difference(marks.get(&(i-1)).unwrap()).cloned().collect();
                // add open preconds to their corresponding layer 
                for precond in open_preconds.iter() {
                    let membership_layer = graphplan.facts.get(&precond).unwrap();
                    match g.get_mut(membership_layer) {
                        Some(set) => {
                            set.insert(precond.clone());
                        },
                        None => {
                            g.insert(*membership_layer, HashSet::from([*precond]));
                        }
                    }
                }
                for add in min_action.add_effects[0].iter() {
                    if let Some(set) = marks.get_mut(&i) {
                        set.insert(add.clone());
                    }
                    if let Some(set) = marks.get_mut(&(i-1)) {
                        set.insert(add.clone());
                    }
                }
            }
        }
        len
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
        let h = FF::calculate_h(&domain, &HashSet::from([0]), &HashSet::from([4]));
        assert_eq!(h, 4.0);
    }
}