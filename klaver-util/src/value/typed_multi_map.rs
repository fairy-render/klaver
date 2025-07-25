use std::cell::RefCell;

use rquickjs::{Ctx, FromJs, IntoJs, array::ArrayIter, class::Trace};

use crate::{
    FromJsIter, Iter, NativeIteratorInterface, Pair, TypedList, TypedListValues, TypedMap,
    TypedMapEntries,
};

pub struct TypedMultiMap<'js, K, T> {
    map: TypedMap<'js, K, TypedList<'js, T>>,
}

impl<'js, K, T> Clone for TypedMultiMap<'js, K, T> {
    fn clone(&self) -> Self {
        TypedMultiMap {
            map: self.map.clone(),
        }
    }
}

impl<'js, K, T> TypedMultiMap<'js, K, T>
where
    K: FromJs<'js> + IntoJs<'js> + Clone,
    T: FromJs<'js> + IntoJs<'js>,
{
    pub fn new(ctx: Ctx<'js>) -> rquickjs::Result<TypedMultiMap<'js, K, T>> {
        Ok(TypedMultiMap {
            map: TypedMap::new(ctx)?,
        })
    }
    pub fn append(&self, ctx: &Ctx<'js>, key: K, value: T) -> rquickjs::Result<()> {
        let list = if let Some(array) = self.map.get(key.clone())? {
            array.clone()
        } else {
            let array = TypedList::new(ctx.clone())?;
            self.map.set(key, array.clone())?;
            array
        };

        list.push(value)?;
        Ok(())
    }

    pub fn set(&self, ctx: &Ctx<'js>, key: K, value: T) -> rquickjs::Result<()> {
        let array = TypedList::new(ctx.clone())?;
        array.push(value)?;
        self.map.set(key, array)?;
        Ok(())
    }

    pub fn get(&self, key: K) -> rquickjs::Result<Option<T>> {
        let Some(i) = self.map.get(key)? else {
            return Ok(None);
        };

        i.get(0)
    }

    pub fn has(&self, key: K) -> rquickjs::Result<bool> {
        self.map.has(key)
    }

    pub fn delete(&self, key: K) -> rquickjs::Result<()> {
        self.map.del(key)
    }

    pub fn get_all(&self, key: K) -> rquickjs::Result<Option<TypedList<'js, T>>> {
        self.map.get(key)
    }

    pub fn entries(&self) -> rquickjs::Result<TypedMultiMapEntries<'js, K, T>> {
        let entries = self.map.entries()?;

        Ok(TypedMultiMapEntries {
            iter: entries,
            queue: RefCell::new(None),
        })
    }
}

impl<'js, K, V> Trace<'js> for TypedMultiMap<'js, K, V> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.map.trace(tracer)
    }
}

pub struct TypedMultiMapEntries<'js, K, T> {
    iter: TypedMapEntries<'js, K, TypedList<'js, T>>,
    queue: RefCell<Option<(K, TypedListValues<'js, T>)>>,
}

impl<'js, K, T> Trace<'js> for TypedMultiMapEntries<'js, K, T> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.iter.trace(tracer);
        if let Some(n) = self.queue.borrow().as_ref() {
            n.1.trace(tracer);
        }
    }
}

impl<'js, K, T> NativeIteratorInterface<'js> for TypedMultiMapEntries<'js, K, T>
where
    K: FromJs<'js> + IntoJs<'js> + Clone,
    T: FromJs<'js> + IntoJs<'js>,
{
    type Item = Pair<K, T>;

    fn next(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Option<Self::Item>> {
        loop {
            let mut queue = self.queue.borrow_mut();
            let Some((key, iter)) = queue.as_mut() else {
                let Some(next) = self.iter.next(ctx)? else {
                    return Ok(None);
                };

                let iter = next.1.values()?;

                *queue = Some((next.0.clone(), iter));

                continue;
            };

            let Some(next) = iter.next(ctx)? else {
                *queue = None;
                continue;
            };

            return Ok(Some(Pair(key.clone(), next)));
        }
    }

    fn returns(&self, ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        self.iter.returns(ctx)
    }
}
