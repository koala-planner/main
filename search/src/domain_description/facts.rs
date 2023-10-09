use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Facts {
    literals: Vec<String>,
    ids: HashMap<String, u32>
}

impl Facts {
    pub fn new(literals: Vec<String>) -> Facts {
        let mut ids = HashMap::new();
        for (i, fact) in literals.iter().cloned().enumerate() {
            ids.insert(fact, i as u32);
        }
        Facts{literals, ids}
    }

    pub fn get_id(&self, fact: &str) -> u32 {
        self.ids[fact]
    }

    pub fn get_fact(&self, id: u32) -> &String {
        &self.literals[id as usize]
    }

    pub fn extend(&self, extension: Vec<String>) -> Facts {
        let mut max_id = self.ids.iter()
            .map(|(literal, id)| {
                id
            }).max().unwrap() + 1;
        let mut new_literals = self.literals.clone();
        let mut new_ids = self.ids.clone();
        for literal in extension.into_iter() {
            new_literals.push(literal.clone());
            new_ids.insert(literal, max_id);
            max_id += 1;
        }
        Facts { literals: new_literals, ids: new_ids }
    }

    pub fn count(&self) -> u32 {
        self.literals.len() as u32
    }

    pub fn contains(&self, literal: &String) -> bool {
        self.literals.contains(literal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn correctness_test() {
        let literals = Vec::from([
            String::from("+at[waypoint0, rover0]"),
            String::from("-at[waypoint0, rover0]"),
            String::from("+at[waypoint1, rover0]"),
            String::from("-at[waypoint1, rover0]")
        ]);
        let facts = Facts::new(literals);
        assert_eq!(facts.get_id("+at[waypoint0, rover0]"), 0);
        assert_eq!(facts.get_id("-at[waypoint0, rover0]"), 1);
        assert_eq!(facts.get_id("+at[waypoint1, rover0]"), 2);
        assert_eq!(facts.get_id("-at[waypoint1, rover0]"), 3);

        assert_eq!(facts.get_fact(0), "+at[waypoint0, rover0]");
        assert_eq!(facts.get_fact(1), "-at[waypoint0, rover0]");
        assert_eq!(facts.get_fact(2), "+at[waypoint1, rover0]");
        assert_eq!(facts.get_fact(3), "-at[waypoint1, rover0]");
    }

    #[test]
    pub fn extension_test() {
        let literals = Vec::from([
            String::from("+at[waypoint0, rover0]"),
            String::from("-at[waypoint0, rover0]"),
            String::from("+at[waypoint1, rover0]"),
            String::from("-at[waypoint1, rover0]")
        ]);
        let facts = Facts::new(literals);
        let extension = Vec::from([
            String::from("Reached_t1"),
            String::from("Reached_t2"),
        ]);
        let facts = facts.extend(extension);
        assert_eq!(facts.get_id("+at[waypoint0, rover0]"), 0);
        assert_eq!(facts.get_id("-at[waypoint0, rover0]"), 1);
        assert_eq!(facts.get_id("+at[waypoint1, rover0]"), 2);
        assert_eq!(facts.get_id("-at[waypoint1, rover0]"), 3);
        assert_eq!(facts.get_id("Reached_t1"), 4);
        assert_eq!(facts.get_id("Reached_t2"), 5);

        assert_eq!(facts.get_fact(0), "+at[waypoint0, rover0]");
        assert_eq!(facts.get_fact(1), "-at[waypoint0, rover0]");
        assert_eq!(facts.get_fact(2), "+at[waypoint1, rover0]");
        assert_eq!(facts.get_fact(3), "-at[waypoint1, rover0]");
        assert_eq!(facts.get_fact(4), "Reached_t1");
        assert_eq!(facts.get_fact(5), "Reached_t2");
    }
}





