use std::{cell::Cell, rc::Rc};

use event_listener::{Event, listener};
use klaver_util::rquickjs::{self, Ctx};

use crate::{async_id::ResourceId, resource::Resource};

pub struct Task {
    resource: ResourceId,
}

pub struct TaskList {
    event: Rc<Event>,
}

impl TaskList {
    pub fn push<'js, T>(
        &self,
        ctx: Ctx<'js>,
        parent: ResourceId,
        resource: T,
    ) -> rquickjs::Result<()>
    where
        T: Resource<'js>,
    {
        let event = self.event.clone();

        let id = ResourceId::new();

        let task = Rc::new(Task {
            resource: id.clone(),
        });

        ctx.clone().spawn(async move {
            loop {
                listener!(event => listener);
            }
        });

        Ok(())
    }
}
