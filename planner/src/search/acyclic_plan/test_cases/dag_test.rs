use std::collections::{HashSet, HashMap};
use std::rc::Rc;
use super::super::SearchResult;
use crate::task_network::Method;

use super::{HTN, PrimitiveAction, CompoundTask, Task, Applicability};
use super::{AOStarSearch, FONDProblem};

#[cfg(test)]
#[test]
pub fn dag_test() {
    use std::collections::BTreeSet;
    use crate::domain_description::{Facts, DomainTasks};

    let p1 = Task::Primitive(PrimitiveAction::new(
        "p1".to_string(),
        1,
        HashSet::from([0]),
        vec![HashSet::from([]), HashSet::from([1])], 
        vec![HashSet::from([]), HashSet::from([0])]
    ));
    let p2 = Task::Primitive(PrimitiveAction::new(
        "p2".to_string(),
        1,
        HashSet::from([1]),
        vec![HashSet::from([0])], 
        vec![HashSet::from([])]
    ));
    let p3 = Task::Primitive(PrimitiveAction::new(
        "p3".to_string(),
        1,
        HashSet::from([0]),
        vec![HashSet::from([1])], 
        vec![HashSet::from([0])]
    ));
    let c1 = Task::Compound(CompoundTask { name: "c1".to_string(), methods: vec![] });
    let mut domain = Rc::new(DomainTasks::new(vec![p1, p2, p3, c1]));
    let m1 = Method::new(
        "m1".to_string(),
        HTN::new(
            BTreeSet::from([1, 2]), 
            vec![(1,2)],
            domain.clone(),
            HashMap::from([(1,2), (2,3)])
        )
    );
    let m2 = Method::new(
        "m2".to_string(),
        HTN::new(
            BTreeSet::from([1]), 
            vec![],
            domain.clone(),
            HashMap::from([(1,1)])
        )
    );
    let domain = domain.add_methods(vec![(3, m1), (3, m2)]);
    let tn = HTN::new(
        BTreeSet::from([1, 2]),
        vec![(1,2)],
        domain.clone(),
        HashMap::from([(1, 0), (2,3)])
    );
    let mut problem = FONDProblem {
        facts: Facts::new(vec![
            "1".to_string(), "2".to_string(), "3".to_string(),
        ]),
        tasks: domain,
        initial_state: HashSet::from([0]),
        init_tn: tn
    };
    problem.collapse_tn();
    let (result, _) = AOStarSearch::run(&problem);
    assert_eq!(result.is_success(), true);
}