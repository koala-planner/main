use std::collections::{HashSet, HashMap};
use std::rc::Rc;
use super::super::SearchResult;
use crate::task_network::Method;

use super::{HTN, PrimitiveAction, CompoundTask, Task, Applicability};
use super::{AOStarSearch, FONDProblem};

#[cfg(test)]
#[test]
pub fn conformant_test() {
    use crate::domain_description::{Facts, DomainTasks};

    let p1 = Rc::new(Task::Primitive(PrimitiveAction::new(
        "p1".to_string(),
        1,
        HashSet::from([1]),
        vec![HashSet::from([2])], 
        vec![HashSet::from([1])]
    )));
    let p2 = Rc::new(Task::Primitive(PrimitiveAction::new(
        "p2".to_string(),
        1,
        HashSet::from([1]),
        vec![HashSet::from([2]), HashSet::from([2, 5])], 
        vec![HashSet::from([3]), HashSet::from([4])]
    )));
    let p3 = Rc::new(Task::Primitive(PrimitiveAction::new(
        "p3".to_string(),
        1,
        HashSet::from([2]),
        vec![HashSet::new(),], 
        vec![HashSet::new(),]
    )));
    let tn = HTN::new(
        HashSet::from([1,2,3]),
        vec![(1,3), (2,3)],
        HashMap::from(
            [(1, p1.clone()), (2,p2.clone()), (3, p3.clone())]
        )
    ).collapse_tn();
    let problem = FONDProblem {
        facts: Facts::new(vec![
            "1".to_string(), "2".to_string(), "3".to_string(), "4".to_string() ,"5".to_string()
        ]),
        tasks: DomainTasks::from_rc_tasks(vec![p1, p2, p3]),
        initial_state: HashSet::from([1]),
        init_tn: tn
    };
    let result = AOStarSearch::run(&problem);
    assert_eq!(result.is_success(), true);
}