use std::time::Duration;

pub struct SearchStats {
    pub max_depth: u16,
    pub search_nodes: u32,
    pub explored_nodes: u32,
    pub seach_time: Duration,
}

impl std::fmt::Display for SearchStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "max depth: {}", self.max_depth);
        writeln!(f, "# of search nodes: {}", self.search_nodes);
        writeln!(f, "# of explored nodes: {}", self.explored_nodes);
        let time = self.seach_time.as_secs_f64();
        let mm = (time / 60.0).trunc();
        let ss = time - (mm * 60.0);
        writeln!(f, "search duration: {}:{}", mm, ss.trunc())
    }
}