use crate::read_json_domain;
use crate::search::AOStarSearch;
#[test]
#[ignore = "reason"]
pub fn recursive_navigation_test() {
    let problem = read_json_domain("result.json");
    let solution = AOStarSearch::run(&problem);
    assert_eq!(solution.is_success(), true);
}