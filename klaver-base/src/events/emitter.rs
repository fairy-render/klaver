use std::collections::HashMap;

use crate::{DynEvent, IntoDynEvent};

use super::listener::NativeListener;

use super::event::EventKey;

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
    fn dispatch(&self, event: DynEvent<'js>) -> rquickjs::Result<()> {
        Ok(())
    }

    fn dispatch_native<T>(&self, ctx: &Ctx<'js>, event: T) -> rquickjs::Result<()>
    where
        T: IntoDynEvent<'js>,
    {
        let event = event.into_dynevent(ctx)?;

        self.dispatch(event.clone())?;

        let Some(listeners) = self.get_listeners().get(&event.ty(ctx)?) else {
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

    fn add_event_listener_native(
        &mut self,
        event_name: EventKey<'js>,
        listener: Function<'js>,
    ) -> rquickjs::Result<()> {
        self.get_listeners_mut()
            .entry(event_name)
            .or_default()
            .push(EventItem {
                callback: Listener::Js(listener),
                once: false,
            });

        Ok(())
    }

    fn add_event_listener(
        this: This<Class<'js, Self>>,
        event_name: EventKey<'js>,
        listener: Function<'js>,
    ) -> rquickjs::Result<()> {
        this.borrow_mut()
            .add_event_listener_native(event_name, listener)
    }

    fn remove_event_listener_native(
        &mut self,
        event_name: EventKey<'js>,
        listener: Function<'js>,
    ) -> rquickjs::Result<()> {
        let Some(listeners) = self.get_listeners_mut().get_mut(&event_name) else {
            return Ok(());
        };

        listeners.retain(|m| m.callback != listener);

        Ok(())
    }

    fn remove_event_listener(
        this: This<Class<'js, Self>>,
        event_name: EventKey<'js>,
        listener: Function<'js>,
    ) -> rquickjs::Result<()> {
        this.borrow_mut()
            .remove_event_listener_native(event_name, listener)
    }

    fn dispatch_event(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        event: DynEvent<'js>,
    ) -> rquickjs::Result<()> {
        this.borrow().dispatch_native(&ctx, event)
    }
}
