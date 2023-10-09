use std::{collections::HashSet, hash::Hash};

use crate::task_network::applicability::Applicability;

#[derive(Debug, PartialEq, Clone)]
pub struct PrimitiveAction{
    pub name: String,
    pub cost: u32,
    pre_cond: HashSet<u32>,
    add_effects: Vec<HashSet<u32>>,
    del_effects: Vec<HashSet<u32>>,
}

impl PrimitiveAction {
    pub fn new(
        name: String,
        cost: u32,
        pre_cond: HashSet<u32>,
        add_effects: Vec<HashSet<u32>>,
        del_effects: Vec<HashSet<u32>>,
    ) -> Self {
        PrimitiveAction {
            name,
            cost,
            pre_cond,
            add_effects,
            del_effects,
        }
    }

    pub fn is_deterministic(&self) -> bool {
        self.add_effects.len() == 1
    }

    pub fn augment(
        &self,
        add_extension: HashSet<u32>,
        del_extension: HashSet<u32>,
        precond_extension: HashSet<u32>
    ) -> PrimitiveAction {
        let mut new_add_effects = self.add_effects.clone();
        let mut new_del_effects = self.del_effects.clone();
        if new_add_effects.len() == 0 {
            new_add_effects = vec![add_extension];
            new_del_effects = vec![del_extension];
        } else {
            for (add, del) in new_add_effects.iter_mut().zip(new_del_effects.iter_mut()) {
                add.extend(add_extension.clone());
                del.extend(del_extension.clone());
            }
        }
        let mut new_precond = self.pre_cond.clone();
        new_precond.extend(precond_extension);
        PrimitiveAction {
            name: self.name.clone(), cost: self.cost, pre_cond: new_precond,
            add_effects: new_add_effects, del_effects: new_del_effects
        }
    }

    pub fn delete_relax(&self) -> PrimitiveAction {
        let n_effects = self.add_effects.len() as u32;
        let mut new_del_effects = vec![];
        for i in 0..n_effects {
            new_del_effects.push(HashSet::new());
        }
        PrimitiveAction {
            name: self.name.clone() + "__delete_relaxed",
            cost: self.cost,
            pre_cond: self.pre_cond.clone(),
            add_effects: self.add_effects.clone(),
            del_effects: new_del_effects
        }
    }

    pub fn determinize(&self) -> Vec<PrimitiveAction> {
        let mut result = vec![];
        let mut counter = 0;
        for (add, del) in self.add_effects.iter().zip(self.del_effects.iter()) {
            let new_action = PrimitiveAction {
                name: self.name.clone() + "__determinized_" + &counter.to_string(),
                cost: self.cost,
                pre_cond: self.pre_cond.clone(),
                add_effects: vec![add.clone()],
                del_effects: vec![del.clone()]
            };
            result.push(new_action);
            counter+=1;
        }
        result
    }
}

impl Applicability for PrimitiveAction {
    fn is_applicable(&self, state: &HashSet<u32>) -> bool
    {
        for condition in self.pre_cond.iter() {
            if !state.contains(condition) {
                return false;
            }
        }
        true
    }

    fn transition(&self, state: &HashSet<u32>) -> Vec<HashSet<u32>> {
        let mut new_states = Vec::new();
        // Action does not have any effect
        if self.add_effects.len() == 0 {
            return vec![state.clone()];
        }
        for (add_eff, del_eff) in self.add_effects.iter().zip(self.del_effects.iter()){
            let mut new_state: HashSet<u32> = state
            .iter()
            .cloned()
            .filter(|x| !del_eff.contains(x))
            .collect();
            for add in add_eff.iter() {
                new_state.insert(add.clone());
            }
            new_states.push(new_state)
        }
        new_states
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn applicability_test() {
        let mut state = HashSet::from([0, 1]);
        let precond = HashSet::from([0, 1]);
        let empty_effects = Vec::from_iter(HashSet::new());
        let action = PrimitiveAction::new(
            "Action1".to_string(),
            1,
            precond,
            empty_effects.clone(),
            empty_effects.clone(),
        );
        assert_eq!(action.is_applicable(&state), true);
        state.insert(2);
        assert_eq!(action.is_applicable(&state), true);
        state.remove(&1);
        assert_eq!(action.is_applicable(&state), false);
    }

    #[test]
    pub fn determinstic_transition_test() {
        let mut state = HashSet::from([0, 1]);
        let precond = HashSet::from([0, 1]);
        let action = PrimitiveAction::new(
            "Action1".to_string(),
            1,
            precond,
            vec![HashSet::from([2])],
            vec![HashSet::from([0])],
        );
        let new_state = action.transition(&state);
        assert_eq!(new_state[0].contains(&2), true);
        assert_eq!(!new_state[0].contains(&0), true);
        assert_eq!(new_state.len(), 1);
    }

    #[test]
    pub fn non_determinstic_transition_test() {
        let mut state = HashSet::from([0, 1]);
        let precond = HashSet::from([0, 1]);
        let action = PrimitiveAction::new(
            "Action1".to_string(),
            1,
            precond,
            vec![HashSet::from([2]), HashSet::from([3,4,5])],
            vec![HashSet::from([0]), HashSet::from([1, 3])],
        );
        let new_states = action.transition(&state);
        assert_eq!(new_states.len(), 2);
        // first outcome
        assert_eq!(new_states[0].contains(&2), true);
        assert_eq!(!new_states[0].contains(&0), true);
        // second outcome
        assert_eq!(new_states[1].contains(&3), true);
        assert_eq!(new_states[1].contains(&4), true);
        assert_eq!(new_states[1].contains(&5), true);
        assert_eq!(!new_states[1].contains(&1), true);
    }

    #[test]
    pub fn determinism_test() {
        let precond = HashSet::from([0, 1]);
        let nd_action = PrimitiveAction::new(
            "NDAction1".to_string(),
            1,
            precond.clone(),
            vec![HashSet::from([2]), HashSet::from([3,4,5])],
            vec![HashSet::from([0]), HashSet::from([1, 3])],
        );
        assert_eq!(nd_action.is_deterministic(), false);
        let action = PrimitiveAction::new(
            "Action1".to_string(),
            1,
            precond,
            vec![HashSet::from([2])],
            vec![HashSet::from([0])],
        );
        assert_eq!(action.is_deterministic(), true);
    }

    #[test]
    pub fn extension_test() {
        let precond = HashSet::from([0, 1]);
        let action = PrimitiveAction::new(
            "NDAction1".to_string(),
            1,
            precond.clone(),
            vec![HashSet::from([2]), HashSet::from([3,4,5])],
            vec![HashSet::from([0]), HashSet::from([1, 3])],
        );
        let new_action = action.augment(
            HashSet::from([4,18,20]),
            HashSet::from([3, 6]),
            HashSet::from([12,9])
        );
        assert_eq!(new_action.is_applicable(&HashSet::from([12, 0, 1])), false);
        assert_eq!(new_action.is_applicable(&HashSet::from([12, 0, 1, 9])), true);
        let transitions = new_action.transition(&HashSet::from([2, 0, 1]));
        for transition in transitions.iter() {
            assert_eq!(transition.contains(&6), false);
            assert_eq!(transition.contains(&18), true);
        }
    }
    #[test]
    pub fn del_relax_test() {
        let action = PrimitiveAction::new(
            "NDAction1".to_string(),
            1,
            HashSet::from([0, 1]),
            vec![HashSet::from([2]), HashSet::from([2,5])],
            vec![HashSet::from([0]), HashSet::from([1, 3])],
        );
        let relaxed = action.delete_relax();
        assert_eq!(relaxed.is_applicable(&HashSet::from([0])), false);
        assert_eq!(relaxed.is_applicable(&HashSet::from([0, 1])), true);
        let transitions = relaxed.transition(&HashSet::from([0, 1, 3]));
        for transition in transitions {
            assert_eq!(transition.contains(&0), true);
            assert_eq!(transition.contains(&1), true);
            assert_eq!(transition.contains(&3), true);
            assert_eq!(transition.contains(&2), true);
        }
    }

    #[test]
    pub fn determinization_test() {
        let action = PrimitiveAction::new(
            "NDAction1".to_string(),
            1,
            HashSet::from([0, 1]),
            vec![HashSet::from([2]), HashSet::from([2,5])],
            vec![HashSet::from([0]), HashSet::from([1, 3])],
        );
        let determinized = action.determinize();
        assert_eq!(determinized.len(), 2);
        let act_1 = &determinized[0];
        let act_2 = &determinized[1];
        assert_eq!(act_1.is_applicable(&HashSet::from([0,1])), true);
        assert_eq!(act_2.is_applicable(&HashSet::from([0,1])), true);
        assert_eq!(act_1.name, "NDAction1__determinized_0");
        assert_eq!(act_2.name, "NDAction1__determinized_1");

        let transition_1 = act_1.transition(&HashSet::from([0,1,2]));
        assert_eq!(transition_1.len(), 1);
        assert_eq!(transition_1[0], HashSet::from([1,2]));

        let transition_2 = act_2.transition(&HashSet::from([0,1,2,3]));
        assert_eq!(transition_2.len(), 1);
        assert_eq!(transition_2[0], HashSet::from([0,2,5]));
    }
}
