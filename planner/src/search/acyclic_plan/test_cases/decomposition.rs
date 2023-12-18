use std::collections::{HashSet, HashMap};
use std::rc::Rc;
use super::SearchResult;
use crate::task_network::Method;

use super::{HTN, PrimitiveAction, CompoundTask, Task, Applicability};
use super::{AOStarSearch, FONDProblem};

#[cfg(test)]
#[test]
pub fn decomposition_test() {
    use std::collections::BTreeSet;

    use crate::domain_description::{Facts, DomainTasks};

    let p1 = Task::Primitive(PrimitiveAction::new(
        "P1".to_string(),
        1,
        HashSet::new(),
        vec![HashSet::from([1])], 
        vec![HashSet::new()]
    ));
    let p2 = Task::Primitive(PrimitiveAction::new(
        "P2".to_string(),
        1,
        HashSet::from([2]),
        vec![HashSet::from([3])], 
        vec![HashSet::new()]
    ));
    let p3 = Task::Primitive(PrimitiveAction::new(
        "P3".to_string(),
        1,
        HashSet::from([3]),
        vec![HashSet::from([2])], 
        vec![HashSet::new()]
    ));
    let p4 = Task::Primitive(PrimitiveAction::new(
        "P4".to_string(),
        1,
        HashSet::from([1]),
        vec![HashSet::from([2]), HashSet::from([3])], 
        vec![HashSet::from([1]), HashSet::from([1])]
    ));
    let t4 = Task::Compound(CompoundTask{
        name: "t4".to_string(),
        methods: vec![] 
    });
    let t3 = Task::Compound(CompoundTask{
        name: "t3".to_string(),
        methods: vec![] 
    });
    let t2 = Task::Compound(CompoundTask{
        name: "t2".to_string(),
        methods: vec![] 
    });
    let t1 = Task::Compound(CompoundTask{
        name: "t1".to_string(),
        methods: vec![] 
    });
    let mut domain = Rc::new(DomainTasks::new(vec![p1, p2, p3, p4, t1, t2, t3, t4]));
    let mut parsed_methods = vec![];
    let t1_m = Method::new(
        "t1_m".to_string(),
        HTN::new(
            BTreeSet::from([1, 4]),
            vec![],
            domain.clone(),
            HashMap::from([(1, domain.get_id("P1")), (4, domain.get_id("t4"))])
        )
    );
    parsed_methods.push((domain.get_id("t1"), t1_m));
    let t2_m = Method::new(
        "t2_m".to_string(),
        HTN::new(
            BTreeSet::from([4, 3]),
            vec![(4,3)],
            domain.clone(),
            HashMap::from([(4, domain.get_id("P4")), (3, domain.get_id("P3"))])
        )
    );
    parsed_methods.push((domain.get_id("t2"), t2_m));
    let t4_m = Method::new(
        "t4_m".to_string(),
        HTN::new(
            BTreeSet::from([2, 3]), 
            vec![],
            domain.clone(),
            HashMap::from([(2, domain.get_id("P2")), (3, domain.get_id("P3"))])
    ));
    parsed_methods.push((domain.get_id("t4"), t4_m));
    let t3_m = Method::new(
        "t3_m".to_string(),
        HTN::new(
            BTreeSet::from([1, 2]),
            vec![(1,2)],
            domain.clone(),
            HashMap::from([(1, domain.get_id("P2")), (2, domain.get_id("P2"))])
        )
    );
    parsed_methods.push((domain.get_id("t3"), t3_m));
    let domain = domain.add_methods(parsed_methods);
    let init_tn = HTN::new(
        BTreeSet::from([1,2,3]),
        vec![(1, 3), (2, 3)],
        domain.clone(),
        HashMap::from([
            (1, domain.get_id("t1")), (2, domain.get_id("t2")), (3, domain.get_id("t3"))
        ])
    );
    let mut problem = FONDProblem {
        facts: Facts::new(vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()]),
        tasks: domain,
        initial_state: HashSet::new(),
        init_tn: init_tn
    };
    problem.collapse_tn();
    let (result, _) = AOStarSearch::run(&problem);
    assert_eq!(result.is_success(), true);
}