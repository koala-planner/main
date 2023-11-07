use crate::read_json_domain;
use crate::search::AOStarSearch;
#[test]
pub fn recursive_navigation_test() {
    let problem = read_json_domain("src/search/fixed_method/test_cases/general_recursion.json");
    let solution = AOStarSearch::run(&problem);
    assert_eq!(solution.is_success(), true);
}