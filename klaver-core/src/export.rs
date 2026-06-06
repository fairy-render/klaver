use rquickjs::{Ctx, IntoAtom, IntoJs, Object, module::Exports};

use crate::value::structured_clone::Registry;

pub trait Exportable<'js> {
    fn export<T>(ctx: &Ctx<'js>, registry: &Registry, target: &T) -> rquickjs::Result<()>
    where
        T: ExportTarget<'js>;
}

pub trait ExportTarget<'js> {
    fn set<K, V>(&self, ctx: &Ctx<'js>, key: K, value: V) -> rquickjs::Result<()>
    where
        K: IntoAtom<'js>,
        V: IntoJs<'js>;
}

impl<'js> ExportTarget<'js> for Object<'js> {
    fn set<K, V>(&self, _ctx: &Ctx<'js>, key: K, value: V) -> rquickjs::Result<()>
    where
        K: IntoAtom<'js>,
        V: IntoJs<'js>,
    {
        Object::set(self, key, value)
    }
}

impl<'js> ExportTarget<'js> for Exports<'js> {
    fn set<K, V>(&self, ctx: &Ctx<'js>, key: K, value: V) -> rquickjs::Result<()>
    where
        K: IntoAtom<'js>,
        V: IntoJs<'js>,
    {
        let atom = key.into_atom(ctx)?;

        self.export(atom.to_string()?, value)?;
        Ok(())
    }
}

#[macro_export]
macro_rules! create_export {
    ($item: ty) => {
        impl<'js> $crate::Exportable<'js> for $item {
            fn export<T>(
                ctx: &rquickjs::Ctx<'js>,
                _registry: &$crate::Registry,
                target: &T,
            ) -> rquickjs::Result<()>
            where
                T: $crate::ExportTarget<'js>,
            {
                target.set(
                    ctx,
                    <$item as rquickjs::class::JsClass<'js>>::NAME,
                    rquickjs::class::Class::<$item>::create_constructor(ctx)?,
                )
            }
        }
    };
}
