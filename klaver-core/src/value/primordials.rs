use crate::{Core, throw};
use rquickjs::{
    Atom, Ctx, FromJs, Function, IntoJs, JsLifetime, Object, Result, atom::PredefinedAtom,
    class::Trace, prelude::IntoArgs,
};

#[derive(JsLifetime, Trace, Clone)]
pub struct BasePrimordials<'js> {
    // // Constructors
    pub constructor_map: Function<'js>,
    pub constructor_weak_map: Function<'js>,
    pub constructor_set: Function<'js>,
    pub constructor_date: Function<'js>,
    pub constructor_regexp: Function<'js>,
    pub constructor_finalization_registry: Function<'js>,
    // pub constructor_date: Constructor<'js>,
    // pub constructor_error: Constructor<'js>,
    // pub constructor_type_error: Constructor<'js>,
    // pub constructor_range_error: Constructor<'js>,
    // pub constructor_regexp: Constructor<'js>,
    // pub constructor_uint8array: Constructor<'js>,
    // pub constructor_array_buffer: Constructor<'js>,
    // pub constructor_proxy: Constructor<'js>,
    // pub constructor_object: Constructor<'js>,
    // pub constructor_bool: Constructor<'js>,
    // pub constructor_number: Constructor<'js>,
    // pub constructor_string: Constructor<'js>,

    // // Prototypes
    // pub prototype_object: Object<'js>,
    // pub prototype_date: Object<'js>,
    // pub prototype_regexp: Object<'js>,
    // pub prototype_set: Object<'js>,
    // pub prototype_map: Object<'js>,
    // pub prototype_weak_map: Object<'js>,
    // pub prototype_error: Object<'js>,

    // // Functions
    // pub function_array_from: Function<'js>,
    // pub function_array_buffer_is_view: Function<'js>,
    // pub function_get_own_property_descriptor: Function<'js>,
    // pub function_parse_int: Function<'js>,
    // pub function_parse_float: Function<'js>,
    // pub function_symbol_for: Function<'js>,

    // // Symbols
    // pub symbol_dispose: Symbol<'js>,
    // pub symbol_async_dispose: Symbol<'js>,

    // // Atoms
    pub atom_entries: Atom<'js>,
    pub atom_keys: Atom<'js>,
}

impl<'js> BasePrimordials<'js> {
    pub fn from_ctx(ctx: &Ctx<'js>) -> Result<Self> {
        let core = Core::from_ctx(ctx)?;
        let primordials: BasePrimordials<'js> = core.borrow().get("primordials")?;
        Ok(primordials)
    }

    pub fn new(ctx: &Ctx<'js>) -> Result<BasePrimordials<'js>> {
        let globals = ctx.globals();

        let constructor_map: Function = globals.get(PredefinedAtom::Map)?;
        let constructor_weak_map: Function = globals.get(PredefinedAtom::WeakMap)?;

        let atom_entries = Atom::from_str(ctx.clone(), "entries")?;
        let atom_keys = Atom::from_str(ctx.clone(), "keys")?;

        let constructor_set: Function = globals.get(PredefinedAtom::Set)?;

        let constructor_date: Function = globals.get(PredefinedAtom::Date)?;

        let constructor_regexp: Function = globals.get(PredefinedAtom::RegExp)?;

        let constructor_finalization_registry: Function =
            ctx.globals().get("FinalizationRegistry")?;

        Ok(BasePrimordials {
            constructor_map,
            constructor_weak_map,
            constructor_set,
            constructor_date,
            constructor_regexp,
            constructor_finalization_registry,
            atom_entries: atom_entries,
            atom_keys: atom_keys,
        })
    }

    pub fn construct_map<A: IntoArgs<'js>>(&self, args: A) -> rquickjs::Result<Object<'js>> {
        unsafe { self.constructor_map.ref_constructor().construct(args) }
    }

    pub fn construct_weak_map<A: IntoArgs<'js>>(&self, args: A) -> rquickjs::Result<Object<'js>> {
        unsafe { self.constructor_weak_map.ref_constructor().construct(args) }
    }

    pub fn construct_set<A: IntoArgs<'js>>(&self, args: A) -> rquickjs::Result<Object<'js>> {
        unsafe { self.constructor_set.ref_constructor().construct(args) }
    }

    pub fn construct_date<A: IntoArgs<'js>>(&self, args: A) -> rquickjs::Result<Object<'js>> {
        unsafe { self.constructor_date.ref_constructor().construct(args) }
    }

    pub fn construct_regexp<A: IntoArgs<'js>>(&self, args: A) -> rquickjs::Result<Object<'js>> {
        unsafe { self.constructor_regexp.ref_constructor().construct(args) }
    }

    pub fn construct_finalization_registry<A: IntoArgs<'js>>(
        &self,
        args: A,
    ) -> rquickjs::Result<Object<'js>> {
        unsafe {
            self.constructor_finalization_registry
                .ref_constructor()
                .construct(args)
        }
    }
}

impl<'js> FromJs<'js> for BasePrimordials<'js> {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        if let Some(obj) = value.as_object() {
            let constructor_map: Function = obj.get("constructorMap")?;
            let constructor_weak_map: Function = obj.get("constructorWeakMap")?;
            let constructor_set: Function = obj.get("constructorSet")?;
            let constructor_date: Function = obj.get("constructorDate")?;
            let constructor_regexp: Function = obj.get("constructorRegExp")?;
            let constructor_finalization_registry: Function =
                obj.get("constructorFinalizationRegistry")?;

            let atom_entries = Atom::from_str(ctx.clone(), "entries")?;
            let atom_keys = Atom::from_str(ctx.clone(), "keys")?;

            Ok(Self {
                constructor_map,
                constructor_weak_map,
                constructor_set,
                constructor_date,
                constructor_regexp,
                constructor_finalization_registry,
                atom_entries,

                atom_keys,
            })
        } else {
            throw!(@type ctx, "Expected object")
        }
    }
}

impl<'js> IntoJs<'js> for BasePrimordials<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let obj = Object::new(ctx.clone())?;
        obj.set("constructorMap", self.constructor_map)?;
        obj.set("constructorWeakMap", self.constructor_weak_map)?;
        obj.set("constructorSet", self.constructor_set)?;
        obj.set("constructorDate", self.constructor_date)?;
        obj.set("constructorRegExp", self.constructor_regexp)?;
        obj.set(
            "constructorFinalizationRegistry",
            self.constructor_finalization_registry,
        )?;
        Ok(obj.into())
    }
}
