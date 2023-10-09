use std::collections::{HashSet, HashMap};
use std::rc::Rc;
use super::SearchResult;
use crate::task_network::Method;

use super::{HTN, PrimitiveAction, CompoundTask, Task, Applicability};
use super::{AOStarSearch, FONDProblem};

#[cfg(test)]
#[test]
pub fn failure_test() {
    use crate::domain_description::{Facts, DomainTasks};

    let p3 = Rc::new(Task::Primitive(PrimitiveAction::new(
        "p3".to_string(), 
        1, 
        HashSet::new(), 
        vec![],
        vec![] 
    )));
    let t1_method = Method::new(
        "m1".to_string(), 
        HTN::new(
            HashSet::from([3]),
            vec![],
            HashMap::from([(3, p3.clone())])
        )
    );
    let t1 = Rc::new(Task::Compound(CompoundTask::new(
        "t1".to_string(),
        vec![t1_method]
    )));
    let p1 = Rc::new(Task::Primitive(PrimitiveAction::new(
        "p1".to_string(), 
        1, 
        HashSet::new(), 
        vec![HashSet::new(),],
        vec![HashSet::from([1]),] 
    )));
    let p2 = Rc::new(Task::Primitive(PrimitiveAction::new(
        "p2".to_string(), 
        1, 
        HashSet::from([1]), 
        vec![HashSet::new(),],
        vec![HashSet::new(),] 
    )));
    let init_tn = HTN::new(
        HashSet::from([1,2,3]),
        vec![(1,2),(2,3)],
        HashMap::from([
            (1, t1.clone()), (2, p1.clone()), (3,p2.clone())
        ])
    ).collapse_tn();
    let problem = FONDProblem {
        facts: Facts::new(vec!["1".to_string()]),
        tasks: DomainTasks::from_rc_tasks(vec![p1, p2, p3, t1]),
        initial_state: HashSet::from([1, 4]),
        init_tn: init_tn
    };
    let result = AOStarSearch::run(&problem);
    assert_eq!(result.is_success(), false);
}