use super::*;
use std::{collections::{HashSet, HashMap}, iter::repeat};
use crate::task_network::Applicability;

pub fn h_add(domain: &ClassicalDomain, state: &HashSet<u32>, goal: &HashSet<u32>) -> f32 {
    let mut facts: HashMap<u32, u32> = state.iter().cloned().zip(repeat(0 as u32)).collect();
    let mut actions: HashMap<usize, u32> = HashMap::new();
    let mut open_goals: HashSet<u32> = goal.difference(state).cloned().collect();
    while !open_goals.is_empty() {
        let mut changed = false;
        let all_facts: HashSet<u32> = facts.keys().cloned().collect();
        for (i, action) in domain.actions.iter().enumerate() {
            if action.is_applicable(&all_facts) && (!actions.contains_key(&i)) {
                let pre_cond = &action.pre_cond;
                let mut action_weight = 1;
                for (id, weight) in facts.iter() {
                    if pre_cond.contains(id) {
                        action_weight += weight
                    }
                }
                actions.insert(i, action_weight);
                for effect in action.add_effects[0].iter() {
                    open_goals.remove(effect);
                    if !facts.contains_key(effect) {
                        facts.insert(*effect, action_weight);
                    }
                }
                changed = true
            }
        }
        if changed == false {
            return f32::INFINITY;
        }
    }
    let mut total_cost = 0;
    for f in goal.iter() {
        total_cost += facts.get(f).unwrap();
    }
    total_cost as f32
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::heuristics::PrimitiveAction;
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
        let h = h_add(&domain, &HashSet::from([0]), &HashSet::from([4, 0]));
        assert_eq!(h, 6.0);
        let h = h_add(&domain, &HashSet::from([0]), &HashSet::from([4, 2]));
        assert_eq!(h, 8.0);
    }
    #[test]
    pub fn safety_test() {
        let domain = generate_domain();
        let h = h_add(&domain, &HashSet::from([0]), &HashSet::from([5]));
        assert_eq!(h, f32::INFINITY);
    }
    #[test]
    pub fn goal_awareness_test() {
        let domain = generate_domain();
        let h = h_add(&domain, &HashSet::from([0]), &HashSet::from([0]));
        assert_eq!(h, 0.0);
    }
}