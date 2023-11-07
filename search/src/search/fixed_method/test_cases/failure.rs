use std::collections::{HashSet, HashMap};
use std::rc::Rc;
use super::SearchResult;
use crate::task_network::Method;

use super::{HTN, PrimitiveAction, CompoundTask, Task, Applicability};
use super::{AOStarSearch, FONDProblem};

#[cfg(test)]
#[test]
pub fn failure_test() {
    use std::collections::BTreeSet;

    use crate::domain_description::{Facts, DomainTasks};

    let p3 = Task::Primitive(PrimitiveAction::new(
        "p3".to_string(), 
        1, 
        HashSet::new(), 
        vec![],
        vec![] 
    ));
    let t1 = Task::Compound(CompoundTask::new(
        "t1".to_string(),
        vec![]
    ));
    let p1 = Task::Primitive(PrimitiveAction::new(
        "p1".to_string(), 
        1, 
        HashSet::new(), 
        vec![HashSet::new(),],
        vec![HashSet::from([1]),] 
    ));
    let p2 = Task::Primitive(PrimitiveAction::new(
        "p2".to_string(), 
        1, 
        HashSet::from([1]), 
        vec![HashSet::new(),],
        vec![HashSet::new(),] 
    ));
    let mut domain = Rc::new(DomainTasks::new(vec![p1,p2,p3,t1]));
    let t1_method = Method::new(
        "m1".to_string(), 
        HTN::new(
            BTreeSet::from([3]),
            vec![],
            domain.clone(),
            HashMap::from([(3, domain.get_id("p3"))])
        )
    );
    domain.add_methods(vec![(domain.get_id("t1"), t1_method)]);
    let init_tn = HTN::new(
        BTreeSet::from([1,2,3]),
        vec![(1,2),(2,3)],
        domain.clone(),
        HashMap::from([
            (1, domain.get_id("t1")), (2, domain.get_id("p1")), (3, domain.get_id("p2"))
        ])
    );
    let mut problem = FONDProblem {
        facts: Facts::new(vec!["1".to_string(), "4".to_string()]),
        tasks: domain,
        initial_state: HashSet::from([1, 4]),
        init_tn: init_tn
    };
    problem.collapse_tn();
    let result = AOStarSearch::run(&problem);
    assert_eq!(result.is_success(), false);
}