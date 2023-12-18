use crate::read_json_domain;
use crate::search;
#[test]
pub fn satelite_fond_domain() {
    let problem = read_json_domain("src/search/fixed_method/test_cases/satelite.json");
    let (result, _) = search::AOStarSearch::run(&problem);
    assert_eq!(result.is_success(), true)
}