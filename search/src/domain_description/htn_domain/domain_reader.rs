use std::collections::{HashMap, HashSet};
use std::fs;
use serde::{Deserialize, Serialize};

use super::FONDProblem;

#[derive(Debug, Deserialize, Serialize)]
struct RawDomain {
    #[serde(rename = "state_features")]
    facts: Vec<String>,
    mutex_groups: Vec<String>,
    #[serde(rename = "further_strict_mutex_groups")]
    further_mutex_groups: Vec<String>,
    #[serde(rename = "further_non_strict_mutex_groups")]
    non_strict_mutex_groups: Vec<String>,
    #[serde(rename = "known_invariants")]
    invariants: Vec<String>,
    actions: HashMap<String, RawAction>,
    initial_state: HashSet<String>,
    goal: Vec<String>,
    initial_abstract_task: String,
    methods: HashMap<String, RawMethod>,
    tasks: Vec<String>
}

#[derive(Debug, Deserialize, Serialize)]
struct RawAction {
    cost: u32,
    precond: Vec<String>,
    effects: Vec<RawEffect>
}

#[derive(Debug, Deserialize, Serialize)]
struct RawEffect {
    add_eff: HashMap<String, Vec<String>>,
    del_eff: HashMap<String, Vec<String>>
}

#[derive(Debug, Deserialize, Serialize)]
struct RawMethod {
    task: String,
    subtasks: Vec<String>,
    orderings: Vec<(u32, u32)>,
}

pub fn read_json_domain(path: &str) -> FONDProblem {
    let istream = fs::read_to_string(path).expect("Unable to read file");
    let domain: RawDomain = serde_json::from_str(&istream).unwrap();
    // Process actions
    let mut actions = Vec::new();
    for (name, body) in domain.actions.into_iter() {
        let effects: Vec <(Vec<String>, Vec<String>)> = body.effects
                    .into_iter()
                    .map(|x|
                        (x.add_eff.get("unconditional").unwrap().clone(),
                        x.del_eff.get("unconditional").unwrap().clone())
                    )
                    .collect();
        let processed = (name, body.precond, effects);
        actions.push(processed);
    }
    // Processed methods
    let mut methods = vec![];
    for (name, method) in domain.methods.into_iter() {
        let processed_m = (name, method.task, method.subtasks, method.orderings);
        methods.push(processed_m);
    }
    FONDProblem::new(
        domain.facts,
        actions,
        methods,
        domain.tasks,
        domain.initial_state,
        domain.initial_abstract_task
    )
}


#[cfg(test)]
mod test {
    use crate::task_network::{Task, CompoundTask};

    use super::*;

    #[test]
    pub fn correct_count_test() {
        let domain = read_json_domain("src/domain_description/htn_domain/test_case.json");
        assert_eq!(domain.facts.count(), 21);
        let facts = [
            "+at_soil_sample[waypoint0]", "+at_rock_sample[waypoint0]",
            "+at[rover0,waypoint0]", "+empty[rover0store]",
            "-at[rover0,waypoint1]", "-at[rover0,waypoint2]",
            "-at[rover0,waypoint3]", "-visited[waypoint0]",
            "-visited[waypoint1]", "-visited[waypoint2]",
            "-visited[waypoint3]", "-empty[rover0store]",
            "+full[rover0store]", "+have_soil_analysis[rover0,waypoint0]",
            "+have_rock_analysis[rover0,waypoint0]","-at[rover0,waypoint0]",
            "+at[rover0,waypoint1]", "+at[rover0,waypoint2]",
            "+at[rover0,waypoint3]", "+calibrated[camera0,rover0]",
            "+have_image[rover0,objective1,low_res]"
        ];
        for fact in facts.iter() {
            domain.facts.get_id(fact);
        }
        let all_tasks = domain.tasks.get_all_tasks();
        assert_eq!(all_tasks.len(), 59);
        let mut prim_counter = 0;
        let mut method_counter = 0;
        for task in all_tasks.iter() {
            if task.is_primitive() {
                prim_counter += 1;
            } else {
                if let Task::Compound(CompoundTask {name: _, methods}) = task.as_ref() {
                    method_counter += methods.len();
                }
            }
        }
        assert_eq!(prim_counter, 46);
        assert_eq!(method_counter, 46);
        assert_eq!(domain.initial_state.len(), 11);
    }
}