mod structs;
mod classical;

use crate::task_network::{HTN, Task, PrimitiveAction, CompoundTask, Applicability};
pub use structs::TDG;
use crate::domain_description::{ClassicalDomain, DomainTasks};

pub use classical::{h_ff, h_add, h_max};

