use core::fmt;
use event_listener::{Event, EventListener};
use klaver_util::rquickjs::{self, FromJs, IntoJs, Value, class::Trace};
use std::{cell::RefCell, collections::HashMap, rc::Rc, usize};

use crate::cell::ObservableCell;

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
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        Ok(AsyncId(value.get()?))
    }
}

pub struct Task {
    parent: AsyncId,
    children: usize,
    shutdown: Rc<ObservableCell<bool>>,
}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("parent", &self.parent)
            .field("children", &self.children)
            .field("shutdown", &self.shutdown.get())
            .finish()
    }
}

#[derive(Debug)]
struct Inner {
    next_id: usize,
    current_id: AsyncId,
    tasks: HashMap<AsyncId, Task>,
    event: Rc<Event>,
}

impl Inner {
    fn notify_shutdown(&self, parent: AsyncId) -> rquickjs::Result<()> {
        let Some(task) = self.tasks.get(&parent) else {
            return Ok(());
        };

        for (id, task) in &self.tasks {
            if task.parent == parent {
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
            current_id: AsyncId(0),
            event: Rc::new(Event::new()),
        })))
    }
}

impl ExecState {
    pub fn dump(&self) {
        println!("{:#?}", &*self.0.borrow());
    }

    pub fn set_current(&self, current: AsyncId) {
        self.0.borrow_mut().current_id = current;
    }

    /*pub fn enter<T, R>(&self, id: AsyncId, func: T) -> R
    where
        T: FnOnce() -> R,
    {
        {
            let mut inner = self.0.borrow_mut();
            if let Some(task) = inner.tasks.get(&id) {
                if let Some(parent) = task.parent {
                    inner.execution_id.push(parent);
                } else {
                    inner.execution_id.push(AsyncId::root());
                }
            } else if let Some(current) = inner.current_id.last().copied() {
                inner.execution_id.push(current);
            } else {
                inner.execution_id.push(AsyncId::root());
            }
            inner.current_id.push(id);
        }
        let ret = func();
        self.0.borrow_mut().current_id.pop();
        self.0.borrow_mut().execution_id.pop();
        ret
    }

    pub async fn enter_async<T, U, R>(&self, id: AsyncId, func: T) -> R
    where
        T: FnOnce() -> U,
        U: Future<Output = R>,
    {
        {
            let mut inner = self.0.borrow_mut();
            if let Some(current) = inner.current_id.last().copied() {
                inner.execution_id.push(current);
            } else if let Some(task) = inner.tasks.get(&id) {
                if let Some(parent) = task.parent {
                    inner.execution_id.push(parent);
                } else {
                    inner.execution_id.push(AsyncId::root());
                }
            } else {
                inner.execution_id.push(AsyncId::root());
            }
            inner.current_id.push(id);
        }

        let ret = func().await;

        self.0.borrow_mut().current_id.pop();
        self.0.borrow_mut().execution_id.pop();
        ret
    }*/

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

    pub fn create_task(&self, parent: Option<AsyncId>, managed: bool) -> AsyncId {
        let id = self.0.borrow().next_id;
        self.0.borrow_mut().next_id += 1;
        let id = AsyncId(id);

        let parent = parent.unwrap_or_else(|| self.0.borrow().current_id);

        let shutdown = if let Some(parent) = self.0.borrow().tasks.get(&parent) {
            parent.shutdown.get()
        } else {
            false
        };

        if !managed {
            // We wanna track the dependency graph,
            // The task isnt managed by garbagecollector
            if let Some(task) = self.0.borrow_mut().tasks.get_mut(&parent) {
                task.children += 1;
            }
        }

        self.0.borrow_mut().tasks.insert(
            id,
            Task {
                parent: parent,
                children: 0,
                shutdown: Rc::new(ObservableCell::new(shutdown)),
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

    pub fn destroy_task(&self, id: AsyncId, managed: bool) {
        let Some(task) = self.0.borrow_mut().tasks.remove(&id) else {
            return;
        };

        if !managed {
            // Managed tasks (by the vm) should not count towards child resource kind
            if let Some(parent) = self.0.borrow_mut().tasks.get_mut(&task.parent) {
                parent.children -= 1;
            }
        }

        self.0.borrow().event.notify(usize::MAX);
    }

    pub fn trigger_async_id(&self) -> AsyncId {
        self.0.borrow().current_id
    }

    pub fn exectution_trigger_id(&self) -> Option<AsyncId> {
        let current_id = self.trigger_async_id();
        self.0.borrow().tasks.get(&current_id).map(|m| m.parent)
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
