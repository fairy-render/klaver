use rquickjs::{
    Class, Ctx,
    atom::PredefinedAtom,
    class::JsClass,
    prelude::{Func, This},
};

use super::native::{NativeIterator, NativeIteratorInterface};

// Trait for implementers
pub trait IterableProtocol<'js>
where
    Self: JsClass<'js> + 'js,
{
    type Iterator: NativeIteratorInterface<'js> + 'js;

    fn create_iterator(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Self::Iterator>;

    fn add_iterable_prototype(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        let Some(proto) = Class::<Self>::prototype(ctx)? else {
            return Ok(());
        };

        proto.set(
            PredefinedAtom::SymbolIterator,
            Func::new(Self::return_iterator),
        )?;

        Ok(())
    }

    fn return_iterator(
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
    ) -> rquickjs::Result<Class<'js, NativeIterator<'js>>> {
        let iterator = this.0.borrow().create_iterator(&ctx)?;

        let iterator = NativeIterator::new(iterator);

        Class::instance(ctx, iterator)
    }
}
