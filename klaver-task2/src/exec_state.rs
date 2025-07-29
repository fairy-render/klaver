use core::fmt;
use event_listener::{Event, EventListener};
use klaver_util::rquickjs::{self, FromJs, IntoJs, Value, class::Trace};
use std::{cell::RefCell, collections::HashMap, rc::Rc, usize};

use crate::{ResourceKind, cell::ObservableCell};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsyncId(usize);

impl AsyncId {
    pub fn root() -> AsyncId {
        AsyncId(0)
    }
}

impl<'js> Trace<'js> for AsyncId {
    fn trace<'a>(&self, _tracer: klaver_util::rquickjs::class::Tracer<'a, 'js>) {}
}

impl<'js> IntoJs<'js> for AsyncId {
    fn into_js(
        self,
        ctx: &klaver_util::rquickjs::Ctx<'js>,
    ) -> klaver_util::rquickjs::Result<klaver_util::rquickjs::Value<'js>> {
        Ok(Value::new_int(ctx.clone(), self.0 as _))
    }
}

impl<'js> FromJs<'js> for AsyncId {
    fn from_js(_ctx: &rquickjs::Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        Ok(AsyncId(value.get()?))
    }
}

pub struct Task {
    parent: AsyncId,
    children: usize,
    shutdown: Rc<ObservableCell<bool>>,
    kind: ResourceKind,
    attached_to: Option<AsyncId>,
}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("parent", &self.parent)
            .field("children", &self.children)
            .field("shutdown", &self.shutdown.get())
            .field("kind", &self.kind)
            .field("parent_resource", &self.attached_to)
            .finish()
    }
}

#[derive(Debug)]
struct Inner {
    next_id: usize,
    trigger_id: AsyncId,
    execution_id: AsyncId,
    tasks: HashMap<AsyncId, Task>,
    event: Rc<Event>,
}

impl Inner {
    fn notify_shutdown(&self, parent: AsyncId) -> rquickjs::Result<()> {
        let Some(task) = self.tasks.get(&parent) else {
            return Ok(());
        };

        for (id, task) in &self.tasks {
            if task.attached_to == Some(parent) {
                self.notify_shutdown(*id)?;
            }
        }
        task.shutdown.set(true);

        Ok(())
    }
}

#[derive(Clone)]
pub struct ExecState(Rc<RefCell<Inner>>);

impl Default for ExecState {
    fn default() -> Self {
        ExecState(Rc::new(RefCell::new(Inner {
            // next_id: 1,
            // current_id: vec![AsyncId(0)],
            // execution_id: vec![AsyncId(0)],
            // tasks,
            tasks: Default::default(),
            next_id: 1,
            trigger_id: AsyncId(0),
            execution_id: AsyncId::root(),
            event: Rc::new(Event::new()),
        })))
    }
}

impl ExecState {
    pub fn dump(&self) {
        println!("{:#?}", &*self.0.borrow());
    }

    pub fn set_current(&self, current: AsyncId) {
        let mut this = self.0.borrow_mut();
        this.execution_id = current;
        if let Some(task) = this.tasks.get(&current) {
            this.trigger_id = task.parent
        } else {
            this.trigger_id = AsyncId::root();
        }
    }

    pub fn listen(&self) -> EventListener {
        self.0.borrow().event.listen()
    }

    pub async fn shutdown(&self, id: AsyncId) -> rquickjs::Result<()> {
        if self.child_count(id) == 0 {
            return Ok(());
        }

        self.0.borrow().notify_shutdown(id)?;

        loop {
            if self.child_count(id) == 0 {
                break;
            }

            let listener = self.0.borrow().event.listen();

            listener.await;
        }

        Ok(())
    }

    // Wait until the task has shutdown status
    pub async fn wait_shutdown(&self, id: AsyncId) -> rquickjs::Result<()> {
        let cell = if let Some(task) = self.0.borrow().tasks.get(&id) {
            task.shutdown.clone()
        } else {
            return Ok(());
        };

        loop {
            if cell.get() {
                break;
            }

            cell.subscribe().await;
        }

        Ok(())
    }

    pub async fn wait_children(&self, id: AsyncId) {
        loop {
            if self.child_count(id) == 0 {
                break;
            }

            let listener = self.0.borrow().event.listen();

            listener.await;
        }
    }

    fn attach_to_parent_native(&self, mut parent: AsyncId) -> Option<AsyncId> {
        loop {
            if let Some(task) = self.0.borrow_mut().tasks.get_mut(&parent) {
                if task.kind.is_native() {
                    task.children += 1;
                    return Some(parent);
                }
                parent = task.attached_to.unwrap_or_else(|| task.parent);
            } else {
                return None;
            }
        }
    }

    pub fn create_task(&self, parent: Option<AsyncId>, kind: ResourceKind) -> AsyncId {
        let id = self.0.borrow().next_id;
        self.0.borrow_mut().next_id += 1;
        let id = AsyncId(id);

        let resolve_parent = parent.unwrap_or_else(|| {
            let borrow = self.0.borrow();
            if borrow.execution_id != AsyncId::root() {
                borrow.execution_id
            } else {
                borrow.trigger_id
            }
        });

        let shutdown = if let Some(parent) = self.0.borrow().tasks.get(&resolve_parent) {
            parent.shutdown.get()
        } else {
            false
        };

        let attached_to = if kind.is_native() {
            self.attach_to_parent_native(resolve_parent)
        } else {
            None
        };

        // if self.exectution_trigger_id()

        // let parent = self.exectution_trigger_id();

        // let parent = if let Some(parent) = parent {
        //     parent
        // } else {
        //     self.exectution_trigger_id()
        // };

        self.0.borrow_mut().tasks.insert(
            id,
            Task {
                parent: resolve_parent,
                children: 0,
                shutdown: Rc::new(ObservableCell::new(shutdown)),
                kind,
                attached_to,
            },
        );

        self.0.borrow().event.notify(usize::MAX);

        id
    }

    pub fn child_count(&self, id: AsyncId) -> usize {
        self.0
            .borrow()
            .tasks
            .get(&id)
            .map(|m| m.children)
            .unwrap_or_default()
    }

    pub fn destroy_task(&self, id: AsyncId) {
        let Some(task) = self.0.borrow_mut().tasks.remove(&id) else {
            return;
        };

        if let Some(attached_to) = task.attached_to {
            if let Some(parent) = self.0.borrow_mut().tasks.get_mut(&attached_to) {
                parent.children -= 1;
            }
        }

        self.0.borrow().event.notify(usize::MAX);
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
}
