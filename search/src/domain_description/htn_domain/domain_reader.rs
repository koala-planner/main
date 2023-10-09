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