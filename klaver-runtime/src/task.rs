use core::fmt;
use std::rc::Rc;

use klaver_util::sync::{Observable, ObservableCell};

use crate::{id::AsyncId, resource::ResourceKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Working,
    Idle,
    Failed,
}

pub struct Task {
    /// The parent task responsible of spawning this tasks
    pub parent: AsyncId,
    /// Number of subtask spawned by this task
    pub children: usize,
    pub state: Rc<ObservableCell<TaskStatus>>,
    /// Kind of resource
    pub kind: ResourceKind,
    /// Nearest native ancester this tasks is attached to
    pub attached_to: Option<AsyncId>,

    pub references: usize,
}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("parent", &self.parent)
            .field("children", &self.children)
            .field("state", &self.state.get())
            .field("kind", &self.kind)
            .field("attached_to", &self.attached_to)
            .finish()
    }
}
