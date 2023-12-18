use crate::{read_json_domain, domain_description::Facts};
use crate::search::AOStarSearch;
use super::*;
use std::rc::Rc;
use crate::domain_description::DomainTasks;
use std::collections::{HashSet, BTreeSet, HashMap};
use crate::task_network::Method;

#[test]
pub fn recursive_navigation_test() {
    let facts = Facts::new(vec![
        "at_w0".to_string(), "at_w1".to_string(), "at_w2".to_string(), "at_w3".to_string()
    ]);
    let nav_abs = Task::Compound(CompoundTask{
        name: "nav_abs".to_string(),
        methods: vec![] 
    });
    let nav_to_w1 = Task::Primitive(PrimitiveAction::new(
        "nav_to_w1".to_string(),
        1,
        HashSet::from([0]),
        vec![HashSet::from([1])], 
        vec![HashSet::from([0])]
    ));
    let nav_to_w2 = Task::Primitive(PrimitiveAction::new(
        "nav_to_w2".to_string(),
        1,
        HashSet::from([1]),
        vec![HashSet::from([2])], 
        vec![HashSet::from([1])]
    ));
    let nav_to_w3 = Task::Primitive(PrimitiveAction::new(
        "nav_to_w3".to_string(),
        1,
        HashSet::from([2]),
        vec![HashSet::from([3])], 
        vec![HashSet::from([2])]
    ));
    let mut domain = Rc::new(DomainTasks::new(vec![nav_to_w1, nav_to_w2, nav_abs, nav_to_w3]));
    let nav_abs_m_1 = Method::new(
        "nav_abs_m_1".to_string(),
        HTN::new(
            BTreeSet::from([1, 2]),
            vec![(1,2)],
            domain.clone(),
            HashMap::from([(1, domain.get_id("nav_abs")), (2, domain.get_id("nav_abs"))])
        )
    );
    let nav_abs_m_2 = Method::new(
        "nav_abs_m_2".to_string(),
        HTN::new(
            BTreeSet::from([1]),
            vec![],
            domain.clone(),
            HashMap::from([(1, domain.get_id("nav_to_w1"))])
        )
    );
    let nav_abs_m_3 = Method::new(
        "nav_abs_m_3".to_string(),
        HTN::new(
            BTreeSet::from([1]),
            vec![],
            domain.clone(),
            HashMap::from([(1, domain.get_id("nav_to_w2"))])
        )
    );
    let parsed_methods = vec![(2, nav_abs_m_1), (2,nav_abs_m_2), (2, nav_abs_m_3)];
    let domain = domain.add_methods(parsed_methods);
    let tn = HTN::new(
        BTreeSet::from([1,2]), 
        vec![(1,2)], 
        domain.clone(), 
        HashMap::from([(1,2), (2,3)])
    );
    let mut problem = FONDProblem {
        facts: facts,
        tasks: domain.clone(),
        initial_state: HashSet::from([0]),
        init_tn: tn
    };
    problem.collapse_tn();
    let (result, _) = AOStarSearch::run(&problem);
    assert_eq!(result.is_success(), true);
}