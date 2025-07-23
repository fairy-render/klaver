use crate::{map::Entry, typed_list::TypedList, typed_map::TypedMap, MapEntries};
use rquickjs::{array::ArrayIter, class::Trace, Ctx, FromJs, IntoJs};

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

    pub fn entries(&self) -> rquickjs::Result<TypedMultiMapIter<'js, K, T>> {
        let entries = self.map.entries()?;

        Ok(TypedMultiMapIter {
            iter: entries,
            queue: None,
        })
    }
}

impl<'js, K, V> Trace<'js> for TypedMultiMap<'js, K, V> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        self.map.trace(tracer)
    }
}

pub struct TypedMultiMapIter<'js, K, T> {
    iter: MapEntries<'js, K, TypedList<'js, T>>,
    queue: Option<(K, ArrayIter<'js, T>)>,
}

impl<'js, K, T> TypedMultiMapIter<'js, K, T>
where
    K: FromJs<'js> + Clone,
    T: FromJs<'js> + IntoJs<'js>,
{
    pub fn next(&mut self) -> rquickjs::Result<Option<(K, T)>> {
        loop {
            let Some((key, iter)) = self.queue.as_mut() else {
                let Some(next) = self.iter.next() else {
                    return Ok(None);
                };

                let (k, v) = next?;
                let iter = v.iter();

                self.queue = Some((k, iter));

                continue;
            };

            let Some(next) = iter.next() else {
                self.queue = None;
                continue;
            };

            let next = next?;

            return Ok(Some((key.clone(), next)));
        }
    }
}

impl<'js, K, T> Iterator for TypedMultiMapIter<'js, K, T>
where
    K: FromJs<'js> + Clone,
    T: FromJs<'js> + IntoJs<'js>,
{
    type Item = rquickjs::Result<Entry<K, T>>;

    fn next(&mut self) -> Option<Self::Item> {
        Self::next(self)
            .map(|m| m.map(|(k, v)| Entry { key: k, value: v }))
            .transpose()
    }
}
