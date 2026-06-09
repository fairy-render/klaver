use rquickjs::{
    Class, Ctx,
    atom::PredefinedAtom,
    class::JsClass,
    prelude::{Func, This},
};

use crate::async_iterator::native::{NativeAsyncIterator, NativeAsyncIteratorInterface};

// Trait for implementers
pub trait AsyncIterableProtocol<'js>
where
    Self: JsClass<'js> + 'js,
{
    type Iterator: NativeAsyncIteratorInterface<'js> + 'js;

    fn create_stream(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Self::Iterator>;

    fn add_iterable_prototype(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        let Some(proto) = Class::<Self>::prototype(ctx)? else {
            return Ok(());
        };

        proto.set(
            PredefinedAtom::SymbolAsyncIterator,
            Func::new(Self::return_iterator),
        )?;

        Ok(())
    }

    fn return_iterator(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<Class<'js, NativeAsyncIterator<'js>>> {
        let iterator = this.0.borrow().create_stream(&ctx)?;

        let iterator = NativeAsyncIterator::new(iterator);

        Class::instance(ctx, iterator)
    }
}
