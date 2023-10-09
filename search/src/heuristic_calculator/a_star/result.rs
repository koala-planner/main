pub enum AStarResult {
    Success(f32),
    NoSolution
}

impl AStarResult {
    pub fn is_success(&self) -> bool {
        match self {
            AStarResult::Success(_) => true,
            _ => false
        }
    }
}