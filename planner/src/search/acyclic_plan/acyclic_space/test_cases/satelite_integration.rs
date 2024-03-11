use crate::read_json_domain;
use crate::search;
use crate::search::acyclic_plan::acyclic_space;
#[test]
pub fn satelite_fond_domain() {
    let problem = read_json_domain("src/search/fixed_method/test_cases/satelite.json");
    let (result, _) = acyclic_space::AOStarSearch::run(&problem, crate::search::acyclic_plan::HeuristicType::HFF);
    assert_eq!(result.is_success(), true)
}