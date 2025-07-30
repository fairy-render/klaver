use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use rquickjs::{Ctx, JsLifetime};

use crate::throw_if;

pub struct Extensions {
    inner: Rc<RefCell<HashMap<TypeId, Box<dyn Any>>>>,
}

unsafe impl<'js> JsLifetime<'js> for Extensions {
    type Changed<'to> = Extensions;
}

impl Extensions {
    pub fn instance(ctx: &Ctx<'_>) -> rquickjs::Result<Extensions> {
        if let Some(found) = ctx.userdata::<Self>() {
            Ok(Extensions {
                inner: found.inner.clone(),
            })
        } else {
            let inner = Rc::new(RefCell::new(HashMap::default()));

            let _ = throw_if!(
                ctx,
                ctx.store_userdata(Extensions {
                    inner: inner.clone()
                })
            );

            Ok(Extensions { inner: inner })
        }
    }

    pub fn get<T: Any + Clone>(&self) -> Option<T> {
        let inner = self.inner.borrow();
        let Some(found) = inner.get(&TypeId::of::<T>()) else {
            return None;
        };
        found.downcast_ref::<T>().cloned()
    }

    pub fn set<T: Any + Clone>(&self, value: T) -> Option<T> {
        let mut inner = self.inner.borrow_mut();
        let Some(found) = inner.insert(TypeId::of::<T>(), Box::new(value)) else {
            return None;
        };
        found.downcast::<T>().map(|m| *m).ok()
    }

    pub fn rm<T: Any + Clone>(&self) -> Option<T> {
        let mut inner = self.inner.borrow_mut();
        let Some(found) = inner.remove(&TypeId::of::<T>()) else {
            return None;
        };
        found.downcast::<T>().map(|m| *m).ok()
    }
}
