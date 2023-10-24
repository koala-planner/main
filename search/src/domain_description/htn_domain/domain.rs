use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::task_network::CompoundTask;
use crate::task_network::Method;

use super::DomainTasks;
use super::{HTN, PrimitiveAction, Facts, Task};
use std::rc::Rc;

#[derive(Debug)]
pub struct FONDProblem {
    pub facts: Facts,
    pub tasks: DomainTasks,
    pub initial_state: HashSet<u32>,
    pub init_tn: HTN,
}

impl FONDProblem {
    pub fn new(literals: Vec<String>,
                // Vector of tuples in the form (action name, preconds, Vec<(Vec<add>, Vec<del>)>)
                actions: Vec<(String, Vec<String>, Vec<(Vec<String>, Vec<String>)>)>,
                // Vector of tuples in the form (method name, task name, vec<subtasks>, vec<orderings>)
                methods: Vec<(String, String, Vec<String>, Vec<(u32, u32)>)>,
                abstract_tasks: Vec<String>,
                init: HashSet<String>,
                first_task: String
    ) -> FONDProblem {
        let facts =  Facts::new(literals);
        let initial_state = init.iter().map(|x| facts.get_id(x)).collect();
        let mut processed_tasks  = Vec::new();
        // Process Tasks
        for (name, precond, effects) in actions.into_iter() {
            let mut add_effs = vec![];
            let mut del_effs = vec![];
            for (add_effect, del_effect) in effects.into_iter() {
                let add_set_i: HashSet<u32> = HashSet::from_iter(add_effect.into_iter()
                    .map(|x| facts.get_id(&x)));
                let del_set_i: HashSet<u32> = HashSet::from_iter(del_effect.into_iter()
                    .map(|x| facts.get_id(&x)));
                add_effs.push(add_set_i);
                del_effs.push(del_set_i);
            }
            let action = PrimitiveAction::new(
                name,
                1,
                precond.into_iter().map(|x| facts.get_id(&x)).collect(),
                add_effs,
                del_effs
            );
            processed_tasks.push(Task::Primitive(action));
        }

        // Process Abstract tasks
        for task in abstract_tasks.into_iter() {
            let new_task = Task::Compound(
                CompoundTask { name: task, methods: vec![] }
            );
            processed_tasks.push(new_task);
        }
        let mut domain_tasks = DomainTasks::new(processed_tasks);

        // Process methods
        for (name, task, subtasks, orderings) in methods.into_iter() {
            let processed_orderings: Vec<(u32, u32)>= orderings.into_iter()
                    .map(|(x, y)| (&subtasks[x as usize], &subtasks[y as usize]))
                    .map(|(x, y)| (domain_tasks.get_id(x), domain_tasks.get_id(y)))
                    .collect();
            let subtasks: BTreeSet<u32> = subtasks.into_iter()
                    .map(|x| domain_tasks.get_id(&x))
                    .collect();
            let mappings: HashMap<u32, Rc<Task>> = subtasks.iter()
                    .cloned()
                    .map(|x| (x, domain_tasks.get_task(x)))
                    .collect();
            let decomposition = HTN::new(subtasks, processed_orderings, mappings);
            let method = Method::new(name, decomposition);
            let task_id = domain_tasks.get_id(&task);
            domain_tasks.add_method(task_id, method);
        }

        // initial abstract task
        let initial_task_id = domain_tasks.get_id(&first_task);
        let tn = HTN::new(
            BTreeSet::from([initial_task_id]),
            vec![],
            HashMap::from([
                (initial_task_id, domain_tasks.get_task(initial_task_id))
            ])
        );
        FONDProblem {
            facts,
            tasks: domain_tasks,
            initial_state,
            init_tn: tn
        }
    }
}