use rquickjs::{
    Class, Ctx, JsLifetime, Object, Value,
    class::{JsClass, Trace},
};
use rquickjs_util::{Inheritable, Inheritable2, throw};

use super::emitter::{Emitter, EventList};

#[rquickjs::class]
#[derive(Trace)]
pub struct EventTarget<'js> {
    listeners: EventList<'js>,
}

unsafe impl<'js> JsLifetime<'js> for EventTarget<'js> {
    type Changed<'to> = EventTarget<'to>;
}

#[rquickjs::methods]
impl<'js> EventTarget<'js> {
    #[qjs(constructor)]
    pub fn new() -> rquickjs::Result<EventTarget<'js>> {
        Ok(EventTarget {
            listeners: Default::default(),
        })
    }
}

impl<'js> EventTarget<'js> {
    // pub fn inherit<T: Emitter<'js>>(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
    //     let event_target = Class::<Self>::prototype(ctx)?;
    //     let proto = Class::<T>::prototype(ctx)?;

    //     let Some(proto) = proto else {
    //         throw!(@type ctx, "Could not get prototype")
    //     };

    //     proto.set_prototype(event_target.as_ref())?;

    //     Ok(())
    // }

    // pub fn instance_of(ctx: &Ctx<'js>, value: &Object<'js>) -> rquickjs::Result<bool> {
    //     let ctor = Class::<Self>::create_constructor(ctx)?;

    //     let Some(ctor) = ctor else { panic!("no ctor") };

    //     Ok(value.is_instance_of(&ctor))
    // }
}

impl<'js, T> Inheritable2<'js, T> for EventTarget<'js> where T: JsClass<'js> {}

impl<'js> Emitter<'js> for EventTarget<'js> {
    fn get_listeners(&self) -> &EventList<'js> {
        &self.listeners
    }

    fn get_listeners_mut(&mut self) -> &mut EventList<'js> {
        &mut self.listeners
    }
}
