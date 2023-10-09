use crate::task_network::Method;

use super::*;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Debug)]
pub struct DomainTasks {
    list: Vec<Rc<Task>>,
    ids: HashMap<String, u32>,
}

impl DomainTasks {
    pub fn new(tasks: Vec<Task>) -> DomainTasks {
        let mut rc_tasks = vec![];
        let mut ids = HashMap::new();
        for (i, task) in tasks.into_iter().enumerate() {
            ids.insert(task.get_name(), i as u32);
            rc_tasks.push(Rc::new(task));
        }
        DomainTasks {
            list: rc_tasks,
            ids: ids,
        }
    }

    pub fn from_rc_tasks(tasks: Vec<Rc<Task>>) -> DomainTasks {
        let mut rc_tasks = vec![];
        let mut ids = HashMap::new();
        for (i, task) in tasks.into_iter().enumerate() {
            ids.insert(task.get_name(), i as u32);
            rc_tasks.push(task);
        }
        DomainTasks {
            list: rc_tasks,
            ids: ids,
        }
    }

    pub fn get_id(&self, task: &str) -> u32 {
        self.ids[task]
    }

    pub fn get_task(&self, id: u32) -> Rc<Task> {
        self.list[id as usize].clone()
    }

    pub fn add_method(&mut self, id: u32, new_method: Method) {
        let mut task = (*self.list[id as usize]).clone();
        let name = task.get_name();
        let mut new_methods = vec![];
        if let Task::Compound(CompoundTask{name, mut methods}) = task {
            new_methods = methods.clone();
            new_methods.push(new_method);
        }
        let new_task = Task::Compound(CompoundTask { name: name, methods: new_methods });
        self.list[id as usize] = Rc::new(new_task);
    }

    pub fn get_all_tasks(&self) -> &Vec<Rc<Task>> {
        &self.list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn correctness_test() {
        let empty = HashSet::new();
        let t1 = Task::Primitive(PrimitiveAction::new(
            "ObtainPermit".to_string(),
            1,
            empty.clone(),
            vec![empty.clone()],
            vec![empty.clone()],
        ));
        let t2 = Task::Primitive(PrimitiveAction::new(
            "HireBuilder".to_string(),
            1,
            empty.clone(),
            vec![empty.clone()],
            vec![empty.clone()],
        ));
        let t3 = Task::Compound(CompoundTask::new("Construct".to_string(), Vec::new()));
        let t4 = Task::Primitive(PrimitiveAction::new(
            "PayBuilder".to_string(),
            1,
            empty.clone(),
            vec![empty.clone()],
            vec![empty.clone()],
        ));
        let tasks = vec![t1,t2,t3,t4];
        let task_defs = DomainTasks::new(tasks);
        assert_eq!(task_defs.get_id("ObtainPermit"), 0);
        assert_eq!(task_defs.get_id("HireBuilder"), 1);
        assert_eq!(task_defs.get_id("Construct"), 2);
        assert_eq!(task_defs.get_id("PayBuilder"), 3);

        assert_eq!(task_defs.get_task(0).get_name(), "ObtainPermit");
        assert_eq!(task_defs.get_task(1).get_name(), "HireBuilder");
        assert_eq!(task_defs.get_task(2).get_name(), "Construct");
        assert_eq!(task_defs.get_task(3).get_name(), "PayBuilder");
    }
}
