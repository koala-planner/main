use super::*;

#[derive(Debug)]
pub enum SearchResult {
    Success(StrongPolicy),
    NoSolution
}

impl SearchResult {
    pub fn is_success(&self) -> bool {
        match self {
            SearchResult::Success(_) => true,
            SearchResult::NoSolution => false
        }
    }
}

impl std::fmt::Display for SearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::NoSolution => write!(f, "Problem has no solution"),
            Self::Success(x) => {
                x.fmt(f)
            }
        }
    }
}
