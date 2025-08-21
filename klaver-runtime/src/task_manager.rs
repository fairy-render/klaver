use core::fmt;
use klaver_util::{
    rquickjs::{self, Ctx, FromJs, IntoJs, Value, class::Trace},
    sync::{Notify, Observable, ObservableCell},
    throw,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc, usize};
use tracing::trace;

use crate::{
    id::AsyncId,
    resource::ResourceKind,
    task::{Task, TaskStatus},
};

#[derive(Debug)]
struct Inner {
    next_id: usize,
    trigger_id: AsyncId,
    execution_id: AsyncId,
    tasks: HashMap<AsyncId, Task>,
    event: Rc<Notify>,
}

#[derive(Clone)]
pub struct TaskManager(Rc<RefCell<Inner>>);

impl Default for TaskManager {
    fn default() -> Self {
        TaskManager(Rc::new(RefCell::new(Inner {
            tasks: Default::default(),
            next_id: 1,
            trigger_id: AsyncId::root(),
            execution_id: AsyncId::root(),
            event: Rc::new(Notify::default()),
        })))
    }
}

impl TaskManager {
    // Get nearest task which is a Root
    // #[allow(unused)]
    // pub fn root(&self, mut id: AsyncId) -> Option<AsyncId> {
    //     loop {
    //         if let Some(task) = self.0.borrow().tasks.get(&id) {
    //             if task.kind == Resour::ROOT {
    //                 return Some(id);
    //             } else if let Some(attached) = task.attached_to {
    //                 id = attached
    //             } else {
    //                 id = task.parent;
    //             }
    //         } else {
    //             return None;
    //         }
    //     }
    // }

    #[allow(unused)]
    pub fn find_parent<T: Fn(&Task) -> bool>(&self, mut id: AsyncId, search: T) -> Option<AsyncId> {
        loop {
            if let Some(task) = self.0.borrow().tasks.get(&id) {
                if search(task) {
                    return Some(id);
                } else if let Some(attached) = task.attached_to {
                    id = attached;
                } else {
                    id = task.parent;
                }
            } else {
                return None;
            }
        }
    }

    /// Set current execution scope
    pub fn set_current(&self, current: AsyncId) {
        let mut this = self.0.borrow_mut();

        trace!(
            id = ?this.execution_id,
            parent = ?this.trigger_id,
            "Leaving async context"
        );

        this.execution_id = current;
        if let Some(task) = this.tasks.get(&current) {
            this.trigger_id = task.parent
        } else {
            this.trigger_id = AsyncId::root();
        }

        trace!(
            id = ?this.execution_id,
            parent = ?this.trigger_id,
            "Enter async context"
        );
    }

    /// Wait for all subtasks to be destroyed
    pub async fn wait_children(&self, id: AsyncId) {
        loop {
            if self.child_count(id) == 0 {
                break;
            }

            trace!(id = %id, "Waiting: {}", self.child_count(id));

            let listener = self.0.borrow().event.listen();

            listener.await;
        }
    }

    pub fn create_task(
        &self,
        parent: Option<AsyncId>,
        kind: ResourceKind,
        persist: bool,
    ) -> AsyncId {
        let id = self.0.borrow().next_id;
        self.0.borrow_mut().next_id += 1;
        let id = AsyncId(id);

        let resolve_parent = parent.unwrap_or_else(|| self.0.borrow().execution_id);

        let attached_to = if persist {
            self.attach_to_parent_native(resolve_parent)
        } else {
            None
        };

        trace!(id = %id, kind = %kind, parent = %resolve_parent, attached_to = ?attached_to, "Create task");

        self.0.borrow_mut().tasks.insert(
            id,
            Task {
                parent: resolve_parent,
                children: 0,
                state: ObservableCell::new(TaskStatus::Working).into(),
                kind,
                attached_to,
                references: 1,
            },
        );

        self.0.borrow().event.notify();

        id
    }

    pub fn task_status(&self, id: AsyncId) -> Option<Rc<ObservableCell<TaskStatus>>> {
        self.0.borrow().tasks.get(&id).map(|m| m.state.clone())
    }

    pub fn child_count(&self, id: AsyncId) -> usize {
        self.0
            .borrow()
            .tasks
            .get(&id)
            .map(|m| m.children)
            .unwrap_or_default()
    }

    /// Remove a task
    pub fn destroy_task(&self, id: AsyncId) -> bool {
        if let Some(task) = self.0.borrow_mut().tasks.get_mut(&id) {
            if task.references > 1 {
                task.references -= 1;
                return false;
            }
        }

        let Some(task) = self.0.borrow_mut().tasks.remove(&id) else {
            return false;
        };

        if let Some(attached_to) = task.attached_to {
            if let Some(parent) = self.0.borrow_mut().tasks.get_mut(&attached_to) {
                parent.children -= 1;
            }
        }

        trace!(id = %id, kind = %task.kind, children = %task.children, attached_to = ?task.attached_to, state = ?task.state.get(), "Destroy task");

        self.0.borrow().event.notify();

        true
    }

    pub fn trigger_async_id(&self) -> AsyncId {
        self.0.borrow().trigger_id
    }

    pub fn exectution_trigger_id(&self) -> AsyncId {
        self.0.borrow().execution_id
    }

    pub fn parent_id(&self, id: AsyncId) -> AsyncId {
        self.0
            .borrow()
            .tasks
            .get(&id)
            .map(|m| m.parent)
            .unwrap_or(AsyncId(0))
    }

    // pub fn task_ctx<'js>(&self, ctx: &Ctx<'js>, id: AsyncId) -> rquickjs::Result<TaskCtx<'js>> {
    //     let mut this = self.0.borrow_mut();
    //     let Some(task) = this.tasks.get_mut(&id) else {
    //         throw!(ctx, "Task not active")
    //     };

    //     task.references += 1;

    //     TaskCtx::new(ctx.clone(), self.clone(), task.kind, id, false)
    // }
}

impl TaskManager {
    fn attach_to_parent_native(&self, mut parent: AsyncId) -> Option<AsyncId> {
        loop {
            if let Some(task) = self.0.borrow_mut().tasks.get_mut(&parent) {
                if task.kind == ResourceKind::ROOT {
                    task.children += 1;
                    return Some(parent);
                }
                parent = task.attached_to.unwrap_or_else(|| task.parent);
            } else {
                return None;
            }
        }
    }
}
