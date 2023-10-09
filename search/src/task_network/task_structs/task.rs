use std::cmp::PartialEq;
use std::fmt;
use std::hash::{Hash, Hasher};

use super::{CompoundTask, PrimitiveAction};

#[derive(Debug, Clone)]
pub enum Task{
    Primitive(PrimitiveAction),
    Compound(CompoundTask),
}

impl PartialEq for Task{
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Primitive(x) => match other {
                Self::Primitive(y) => x.name == y.name,
                Self::Compound(_) => false,
            },
            Self::Compound(x) => match other {
                Self::Primitive(_) => false,
                Self::Compound(y) => x.name == y.name,
            },
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(&other)
    }
}

impl Eq for Task {}

impl Hash for Task {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        match self {
            Task::Compound(x) => x.name.hash(hasher),
            Task::Primitive(x) => x.name.hash(hasher),
        }
    }
}

impl Task {
    pub fn get_name(&self) -> String {
        match self {
            Task::Compound(CompoundTask { name, .. }) => name.clone(),
            Task::Primitive(PrimitiveAction { name, .. }) => name.clone()
        }
    }

    pub fn is_primitive(&self) -> bool {
        match self {
            Task::Primitive(_) => true,
            _ => false
        }
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "{}", self.get_name())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::task_network::{CompoundTask, PrimitiveAction};

    use super::Task;
    #[test]
    pub fn task_name_test() {
        let compound = Task::Compound(CompoundTask::new("task1".to_string(), Vec::new()));
        assert_eq!(compound.get_name(), "task1".to_string());
        let empty_effects = Vec::from_iter(HashSet::new());
        let primitive = Task::Primitive(PrimitiveAction::new(
            "task2".to_string(),
            1,
            HashSet::new(),
            empty_effects.clone(),
            empty_effects.clone()
        ));
        assert_eq!(primitive.get_name(), "task2".to_string());
    }
}