use crate::task_network::Method;

use super::*;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak, self};
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct DomainTasks {
    list: Vec<RefCell<Task>>,
    ids: HashMap<String, u32>,
}

impl DomainTasks {
    pub fn new(tasks: Vec<Task>) -> DomainTasks {
        let mut task_list = vec![];
        let mut ids = HashMap::new();
        for (i, task) in tasks.into_iter().enumerate() {
            ids.insert(task.get_name(), i as u32);
            task_list.push(RefCell::new(task));
        }
        DomainTasks {
            list: task_list,
            ids: ids,
        }
    }

    pub fn get_id(&self, task: &str) -> u32 {
        self.ids[task]
    }

    pub fn get_task(&self, id: u32) -> &RefCell<Task> {
        &self.list[id as usize]
    }

    pub fn count_tasks(&self) -> u32 {
        self.list.len() as u32
    }

    pub fn add_methods(&self, methods: Vec<(u32, Method)>) -> Rc<DomainTasks> {
        let mut new_domain = self.clone();
        for (task_id, method) in methods {
            let mut task = new_domain.list[task_id as usize].borrow().clone();
            let name = task.get_name();
            let mut new_methods = vec![];
            if let Task::Compound(CompoundTask{name, mut methods}) = task {
                new_methods = methods.clone();
                new_methods.push(method);
            } else {
                panic!("{} is not Compound", task_id);
            }
            let new_task = Task::Compound(CompoundTask { name: name, methods: new_methods });
            new_domain.list[task_id as usize] = RefCell::new(new_task);
        }
        let rc_domain = Rc::new(new_domain);
        for t in rc_domain.list.iter() {
            match &mut *t.borrow_mut() {
                Task::Compound(CompoundTask { name, methods }) => {
                    for m in methods.iter_mut() {
                        m.decomposition.change_domain(rc_domain.clone());
                    }
                },
                _ => {}
            }
        }
        rc_domain
    }

    pub fn add_task(&mut self, task: Task) {
        self.ids.insert(task.get_name(), self.list.len() as u32);
        self.list.push(RefCell::new(task));
    }

    pub fn get_all_tasks(&self) -> &Vec<RefCell<Task>>{
        &self.list
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

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

        assert_eq!(task_defs.get_task(0).borrow().get_name(), "ObtainPermit");
        assert_eq!(task_defs.get_task(1).borrow().get_name(), "HireBuilder");
        assert_eq!(task_defs.get_task(2).borrow().get_name(), "Construct");
        assert_eq!(task_defs.get_task(3).borrow().get_name(), "PayBuilder");
    }

    #[test]
    pub fn add_task_test() {
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
        let mut task_defs = DomainTasks::new(tasks);
        let t5 = Task::Compound(CompoundTask::new("ADDED_TASK".to_string(), Vec::new()));
        task_defs.add_task(t5);
        assert_eq!(task_defs.get_id("ObtainPermit"), 0);
        assert_eq!(task_defs.get_id("HireBuilder"), 1);
        assert_eq!(task_defs.get_id("Construct"), 2);
        assert_eq!(task_defs.get_id("PayBuilder"), 3);
        assert_eq!(task_defs.get_id("ADDED_TASK"), 4);

        assert_eq!(task_defs.get_task(0).borrow().get_name(), "ObtainPermit");
        assert_eq!(task_defs.get_task(1).borrow().get_name(), "HireBuilder");
        assert_eq!(task_defs.get_task(2).borrow().get_name(), "Construct");
        assert_eq!(task_defs.get_task(3).borrow().get_name(), "PayBuilder");
        assert_eq!(task_defs.get_task(4).borrow().get_name(), "ADDED_TASK");
    }

    #[test]
    pub fn add_methods_test() {
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
        let t5 = Task::Compound(CompoundTask::new("abstract_t".to_string(), Vec::new()));
        let tasks = vec![t1,t2,t3,t4,t5];
        let task_defs = Rc::new(DomainTasks::new(tasks));
        let t3_m = Method::new(format!("t3_m"), HTN::new(
            BTreeSet::from([1,2]), vec![(1,2)], task_defs.clone(), 
            HashMap::from([(1, 0), (2, 2)]))
        );
        let t5_m1 = Method::new(format!("t5_m1"), HTN::new(
            BTreeSet::from([1]), vec![], task_defs.clone(), 
            HashMap::from([(1, 4)]))
        );
        let t5_m2 = Method::new(format!("t5_m2"), HTN::new(
            BTreeSet::from([1]), vec![], task_defs.clone(), 
            HashMap::from([(1, 1)]))
        );
        let task_defs = task_defs.add_methods(vec![(2, t3_m), (4, t5_m1), (4, t5_m2)]); 
        assert_eq!(task_defs.get_id("ObtainPermit"), 0);
        assert_eq!(task_defs.get_id("HireBuilder"), 1);
        assert_eq!(task_defs.get_id("Construct"), 2);
        assert_eq!(task_defs.get_id("PayBuilder"), 3);
        assert_eq!(task_defs.get_id("abstract_t"), 4);

        match &*task_defs.get_task(2).borrow() {
            Task::Compound(CompoundTask { name, methods }) => {
                assert_eq!(methods.len(), 1);
                let m = &methods[0];
                assert_eq!(m.decomposition.get_nodes().len(), 2);
                assert_eq!(m.decomposition.get_task(1).borrow().get_name(), String::from("ObtainPermit"));
                assert_eq!(m.decomposition.get_task(2).borrow().get_name(), String::from("Construct"));
            },
            _ => panic!("task is not compound")
        }

        match &*task_defs.get_task(4).borrow() {
            Task::Compound(CompoundTask { name, methods }) => {
                assert_eq!(methods.len(), 2);
                let m1 = &methods[0];
                assert_eq!(m1.decomposition.get_nodes().len(), 1);
                assert_eq!(m1.decomposition.get_task(1).borrow().get_name(), String::from("abstract_t"));

                let m2 = &methods[1];
                assert_eq!(m2.decomposition.get_nodes().len(), 1);
                assert_eq!(m2.decomposition.get_task(1).borrow().get_name(), String::from("HireBuilder"));
            },
            _ => panic!("task is not compound")
        }

        assert_eq!(task_defs.get_task(0).borrow().get_name(), "ObtainPermit");
        assert_eq!(task_defs.get_task(1).borrow().get_name(), "HireBuilder");
        assert_eq!(task_defs.get_task(2).borrow().get_name(), "Construct");
        assert_eq!(task_defs.get_task(3).borrow().get_name(), "PayBuilder");
        assert_eq!(task_defs.get_task(4).borrow().get_name(), "abstract_t");
    }
}
