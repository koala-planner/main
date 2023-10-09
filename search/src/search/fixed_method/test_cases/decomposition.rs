use std::collections::{HashSet, HashMap};
use std::rc::Rc;
use super::SearchResult;
use crate::task_network::Method;

use super::{HTN, PrimitiveAction, CompoundTask, Task, Applicability};
use super::{AOStarSearch, FONDProblem};

#[cfg(test)]
#[test]
pub fn decomposition_test() {
    use crate::domain_description::{Facts, DomainTasks};

    let p1 = Rc::new(Task::Primitive(PrimitiveAction::new(
        "P1".to_string(),
        1,
        HashSet::new(),
        vec![HashSet::from([1])], 
        vec![HashSet::new()]
    )));
    let p2 = Rc::new(Task::Primitive(PrimitiveAction::new(
        "P2".to_string(),
        1,
        HashSet::from([2]),
        vec![HashSet::from([3])], 
        vec![HashSet::new()]
    )));
    let p3 = Rc::new(Task::Primitive(PrimitiveAction::new(
        "P3".to_string(),
        1,
        HashSet::from([3]),
        vec![HashSet::from([2])], 
        vec![HashSet::new()]
    )));
    let p4 = Rc::new(Task::Primitive(PrimitiveAction::new(
        "P4".to_string(),
        1,
        HashSet::from([1]),
        vec![HashSet::from([2]), HashSet::from([3])], 
        vec![HashSet::from([1]), HashSet::from([1])]
    )));
    let t4 = Rc::new(Task::Compound(CompoundTask{
        name: "t4".to_string(),
        methods: vec![
            Method::new(
                "t4_m".to_string(),
                HTN::new(HashSet::from([2, 3]), vec![], HashMap::from([
                    (2, Rc::clone(&p2)), (3, Rc::clone(&p3))
                ]))
            )
        ] 
    }));
    let t3 = Rc::new(Task::Compound(CompoundTask{
        name: "t3".to_string(),
        methods: vec![
            Method::new(
                "t3_m".to_string(),
                HTN::new(
                    HashSet::from([1, 2]),
                    vec![(1,2)],
                    HashMap::from([
                        (1, Rc::clone(&p2)), (2, Rc::clone(&p2))
                    ])
                )
            )
        ] 
    }));
    let t2 = Rc::new(Task::Compound(CompoundTask{
        name: "t2".to_string(),
        methods: vec![
            Method::new(
                "t2_m".to_string(),
                HTN::new(
                    HashSet::from([4, 3]),
                    vec![(4,3)],
                    HashMap::from([
                        (4, Rc::clone(&p4)), (3, Rc::clone(&p3))
                    ])
                )
            )
        ] 
    }));
    let t1 = Rc::new(Task::Compound(CompoundTask{
        name: "t1".to_string(),
        methods: vec![
            Method::new(
                "t1_m".to_string(),
                HTN::new(
                    HashSet::from([1, 4]),
                    vec![],
                    HashMap::from([
                        (1, Rc::clone(&p1)), (4, Rc::clone(&t4))
                    ])
                )
            )
        ] 
    }));
    let init_tn = HTN::new(
        HashSet::from([1,2,3]),
        vec![(1, 3), (2, 3)],
        HashMap::from([
            (1, Rc::clone(&t1)), (2, Rc::clone(&t2)), (3, Rc::clone(&t3))
        ])
    ).collapse_tn();
    let problem = FONDProblem {
        facts: Facts::new(vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()]),
        tasks: DomainTasks::from_rc_tasks(vec![p1, p2, p3, p4, t1, t2, t3, t4]),
        initial_state: HashSet::new(),
        init_tn: init_tn
    };
    let result = AOStarSearch::run(&problem);
    assert_eq!(result.is_success(), true);
}