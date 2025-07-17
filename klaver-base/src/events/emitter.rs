use std::collections::HashMap;

use super::listener::NativeListener;

use super::event::{Event, EventKey};

use super::listener::Listener;
use rquickjs::class::{JsClass, Trace};
use rquickjs::prelude::{Func, This};
use rquickjs::{Class, Ctx, Function};

#[derive(Trace)]
pub struct EventItem<'js> {
    pub callback: Listener<'js>,
    pub once: bool,
}

pub type EventList<'js> = HashMap<EventKey<'js>, Vec<EventItem<'js>>>;

pub trait Emitter<'js>
where
    Self: JsClass<'js> + Sized + 'js,
{
    fn get_listeners(&self) -> &EventList<'js>;
    fn get_listeners_mut(&mut self) -> &mut EventList<'js>;

    fn add_native_listener<T>(&mut self, event_name: EventKey<'js>, listener: T)
    where
        T: NativeListener<'js> + 'js,
    {
        self.get_listeners_mut()
            .entry(event_name)
            .or_default()
            .push(EventItem {
                callback: Listener::Native(Box::new(listener)),
                once: false,
            });
    }

    #[allow(unused)]
    fn dispatch(&self, event: Class<'js, Event<'js>>) -> rquickjs::Result<()> {
        Ok(())
    }

    fn dispatch_inner(&self, ctx: Ctx<'js>, event: Class<'js, Event<'js>>) -> rquickjs::Result<()> {
        self.dispatch(event.clone())?;

        let Some(listeners) = self.get_listeners().get(&event.borrow().ty) else {
            return Ok(());
        };

        for listener in listeners {
            listener.callback.call(ctx.clone(), event.clone())?;
        }

        Ok(())
    }

    fn add_event_target_prototype(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        let proto = Class::<Self>::prototype(ctx)?.expect("EventEmitter.prototype");
        proto.set("addEventListener", Func::new(Self::add_event_listener))?;
        proto.set(
            "removeEventListener",
            Func::new(Self::remove_event_listener),
        )?;
        proto.set("dispatchEvent", Func::new(Self::dispatch_event))?;

        Ok(())
    }

    fn add_event_listener(
        this: This<Class<'js, Self>>,
        event_name: EventKey<'js>,
        listener: Function<'js>,
    ) -> rquickjs::Result<()> {
        this.0
            .borrow_mut()
            .get_listeners_mut()
            .entry(event_name)
            .or_default()
            .push(EventItem {
                callback: Listener::Js(listener),
                once: false,
            });

        Ok(())
    }

    fn remove_event_listener(
        this: This<Class<'js, Self>>,
        event_name: EventKey<'js>,
        listener: Function<'js>,
    ) -> rquickjs::Result<()> {
        let mut this = this.0.borrow_mut();
        let Some(listeners) = this.get_listeners_mut().get_mut(&event_name) else {
            return Ok(());
        };

        listeners.retain(|m| m.callback != listener);

        Ok(())
    }

    fn dispatch_event(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        event: Class<'js, Event<'js>>,
    ) -> rquickjs::Result<()> {
        this.borrow().dispatch_inner(ctx, event)
    }
}
