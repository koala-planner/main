use std::collections::{HashSet, LinkedList, HashMap};
use crate::domain_description::ClassicalDomain;

#[derive(Debug, Clone)]
pub struct GraphPlan<'a>{
    pub actions: HashMap<usize, u32>,
    pub facts: HashMap<u32, u32>,
    pub depth: u32,
    domain: &'a ClassicalDomain
}

impl<'a> GraphPlan <'a>{
    // returns the membership index for alternating facts and actions layer as a
    // tuple ((action_id->first occurance layer number), (fact_id -> first occurance index))
    // returns None if there is no solution
    pub fn build_graph(domain: &'a ClassicalDomain, state: &HashSet<u32>, goal: &HashSet<u32>)
        -> Option<GraphPlan<'a>>{
        // initiate first action occurance index in the graphplan to infinity
        // mapping is of the form (action id -> (precond_counter, first occurance) 
        let mut action_membership: HashMap<usize, (u32, u32)> = HashMap::new();
        for (i, action) in domain.actions.iter().enumerate() {
            action_membership.insert(i, (0, u32::MAX));
        }
        // initiate first fact occurance index in the graphplan to infinity or 0 if in state
        let mut fact_membership = HashMap::new();
        for fact_id in domain.facts.get_all_ids() {
            if state.contains(&fact_id) {
                fact_membership.insert(fact_id, 0);
            } else {
                fact_membership.insert(fact_id, u32::MAX);
            }
        }
        // iterate untill all goals are satisified
        let mut layer_num = 0;
        let mut layer_facts = state.clone();
        while !GraphPlan::all_goals_satisfied(&fact_membership, goal) {
            // no new facts have been generated
            if layer_facts.len() == 0 {
                return None;
            }
            layer_num += 1;
            let mut layer_actions = vec![];
            // Check which actions will be added to this layer
            for (i, action) in domain.actions.iter().enumerate() {
                if action_membership.get(&i).unwrap().1 != u32::MAX {
                    continue;
                } else {
                    for new_fact in layer_facts.iter() {
                        if action.pre_cond.contains(new_fact) {
                            let precond_counter = action_membership.get(&i).unwrap().0 + 1;
                            action_membership.get_mut(&i).unwrap().0 = precond_counter;
                            if precond_counter == action.pre_cond.len() as u32 {
                                layer_actions.push(i);
                            }
                        }
                    }
                }
            }
            // add scheduled actions
            layer_facts = HashSet::new();
            for layer_action in layer_actions.iter() {
                action_membership.get_mut(layer_action).unwrap().1 = layer_num;
                let effects = &domain.actions[*layer_action].add_effects;
                if effects.len() > 1 {
                    panic!("actions are not all outcome determinized");
                }
                if effects.len() == 0 {
                    continue;
                }
                // compute achieved facts of this state
                for effect in effects[0].iter() {
                    let fact_index = fact_membership.get(effect).unwrap();
                    if *fact_index == u32::MAX {
                        layer_facts.insert(*effect);
                    }
                } 
            }
            // add scheduled facts
            layer_num += 1;
            for fact in layer_facts.iter() {
                let fact_index = fact_membership.get_mut(&fact).unwrap();
                *fact_index = layer_num;
            }
        }
        let actions = HashMap::from(
            action_membership
            .iter()
            .map(|(k,(c, i))| (k.clone(),i.clone()))
            .collect::<HashMap<usize, u32>>()
        );
        Some(GraphPlan { actions: actions, facts: fact_membership, depth: layer_num, domain: domain})
    }

    fn all_goals_satisfied(indices: &HashMap<u32, u32>, goal: &HashSet<u32>) -> bool {
        for fact in goal.iter() {
            let val = indices.get(fact).unwrap();
            if *val == u32::MAX {
                return false;
            }
        }
        return true;
    }

    // computes the goals that are satisfied in each layer
    pub fn compute_goal_indices(&self, goal: &HashSet<u32>) -> HashMap<u32, HashSet<u32>> {
        let indices: Vec<(u32, u32)> = self.facts.iter()
            .filter(|(id, index)| goal.contains(&id))
            .map(|(id, index)| (id.clone(), index.clone())).collect();
        let mut mapping: HashMap<u32, HashSet<u32>> = HashMap::new();
        for (g, index) in indices {
            if mapping.contains_key(&index) {
                if let Some(val) = mapping.get_mut(&index) {
                    val.insert(g);
                }
            } else {
                mapping.insert(index, HashSet::from([g]));
            }
        }
        mapping
    }

    pub fn get_action_layer(&self, index: u32) -> HashSet<usize> {
        if index % 2 == 0 {
            panic!("actions are odd layers")
        }
        self.actions.iter().filter(|(id, number)| {
            **number == index
        }).map(|(id, number)| id).cloned().collect()
    }

    pub fn get_fact_layer(&self, index: u32) -> HashSet<u32> {
        if index % 2 == 1 {
            panic!("facts are even layer")
        }
        self.facts.iter().filter(|(id, layer)| {
            **layer == index
        }).map(|(id, layer)| id).cloned().collect()
    }
}

impl <'a> std::fmt::Display for GraphPlan<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "digraph G {{\n\trankdir=\"LR\"\n");
        for layer in 0..self.depth {
            if layer % 2 == 0 {
                let ids = self.get_fact_layer(layer);
                let facts: Vec<&String> = ids.iter().map(|x| {
                        self.domain.facts.get_fact(*x)
                    }).collect();
                write!(f, "\tsubgraph cluster{} {{\n\t\tlabel=\"layer{}\"\n\t\tstyle=filled;\n\t\tcolor=lightgrey;\n", layer, layer);
                for (id, fact) in ids.iter().zip(facts.iter()) {
                    write!(f, "\t\t{} [label=\"{}\",shape=box]\n", id, fact);
                }
            } else {
                let ids = self.get_action_layer(layer);
                let actions: Vec<&String> = ids.iter().map(|x| {
                    &self.domain.actions[*x].name
                }).collect();
                write!(f, "\tsubgraph cluster{} {{\n\t\tlabel=\"layer{}\"\n", layer, layer);
                for (id, action) in ids.iter().zip(actions.iter()) {
                    write!(f, "\t\t{} [label=\"{}\",shape=box]\n", id, action);
                }
            }
            write!(f, "\t}}\n");
        }
        write!(f, "}}\n");
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::{heuristic_calculator::PrimitiveAction, domain_description::Facts};
    
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
    pub fn graph_correctness_test() {
        let domain = generate_domain();
        let graphplan = GraphPlan::build_graph(&domain, &HashSet::from([0]), &HashSet::from([4])).unwrap();
        let (actions, facts) = (graphplan.actions.clone(), graphplan.facts.clone());
        for action_id in 0..actions.len() {
            assert_eq!(actions.contains_key(&(action_id as usize)), true)
        }
        for fact_id in 0..facts.len() {
            assert_eq!(facts.contains_key(&(fact_id as u32)), true)
        }
        assert_eq!(*actions.get(&0).unwrap(), 1);
        assert_eq!(*actions.get(&1).unwrap(), 3);
        assert_eq!(*actions.get(&2).unwrap(), 3);
        assert_eq!(*actions.get(&3).unwrap(), 5);
        assert_eq!(*facts.get(&0).unwrap(), 0);
        assert_eq!(*facts.get(&1).unwrap(), 2);
        assert_eq!(*facts.get(&2).unwrap(), 4);
        assert_eq!(*facts.get(&3).unwrap(), 4);
        assert_eq!(*facts.get(&4).unwrap(), 6);
        assert_eq!(graphplan.depth, 6);
        assert_eq!(graphplan.get_action_layer(1), HashSet::from([0]));
        assert_eq!(graphplan.get_action_layer(3), HashSet::from([1, 2]));
        assert_eq!(graphplan.get_action_layer(5), HashSet::from([3]));
    }

    #[test]
    pub fn termination_test() {
        let mut domain = generate_domain();
        domain.actions[3].add_effects = vec![];
        let graphplan = GraphPlan::build_graph(&domain, &HashSet::from([0]), &HashSet::from([4])); 
        assert_eq!(graphplan.is_none(), true);
    }
}