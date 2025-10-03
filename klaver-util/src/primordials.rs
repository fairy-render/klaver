use rquickjs::{
    Atom, Ctx, Function, JsLifetime, Object, Result, String, atom::PredefinedAtom, class::Trace,
    prelude::IntoArgs,
};

// use crate::class::CUSTOM_INSPECT_SYMBOL_DESCRIPTION;

// #[derive(JsLifetime)]
// pub struct BasePrimordials<'js> {
//     // Constructors
//     pub constructor_map: Constructor<'js>,
//     pub constructor_weak_map: Constructor<'js>,
//     pub constructor_set: Constructor<'js>,
//     pub constructor_date: Constructor<'js>,
//     pub constructor_error: Constructor<'js>,
//     pub constructor_type_error: Constructor<'js>,
//     pub constructor_range_error: Constructor<'js>,
//     pub constructor_regexp: Constructor<'js>,
//     pub constructor_uint8array: Constructor<'js>,
//     pub constructor_array_buffer: Constructor<'js>,
//     pub constructor_proxy: Constructor<'js>,
//     pub constructor_object: Constructor<'js>,
//     pub constructor_bool: Constructor<'js>,
//     pub constructor_number: Constructor<'js>,
//     pub constructor_string: Constructor<'js>,

//     // Prototypes
//     pub prototype_object: Object<'js>,
//     pub prototype_date: Object<'js>,
//     pub prototype_regexp: Object<'js>,
//     pub prototype_set: Object<'js>,
//     pub prototype_map: Object<'js>,
//     pub prototype_weak_map: Object<'js>,
//     pub prototype_error: Object<'js>,

//     // Functions
//     pub function_array_from: Function<'js>,
//     pub function_array_buffer_is_view: Function<'js>,
//     pub function_get_own_property_descriptor: Function<'js>,
//     pub function_parse_int: Function<'js>,
//     pub function_parse_float: Function<'js>,
//     pub function_symbol_for: Function<'js>,

//     // Symbols
//     pub symbol_dispose: Symbol<'js>,
//     pub symbol_async_dispose: Symbol<'js>,

//     // Atoms
//     pub atom_entries: Atom<'js>,
//     pub atom_keys: Atom<'js>,
// }

// impl<'js> BasePrimordials<'js> {
//     pub fn get<'a>(ctx: &'a Ctx<'js>) -> Result<UserDataGuard<'a, Self>>
//     where
//         Self: Sized + JsLifetime<'js>,
//     {
//         if let Some(primordials) = ctx.userdata::<Self>() {
//             return Ok(primordials);
//         }

//         let primoridals = Self::new(ctx)?;

//         _ = ctx.store_userdata(primoridals)?;
//         Ok(ctx.userdata::<Self>().unwrap())
//     }

//     fn new(ctx: &Ctx<'js>) -> Result<Self> {
//         let globals = ctx.globals();

//         let constructor_object: Constructor = globals.get(PredefinedAtom::Object)?;
//         let prototype_object: Object = constructor_object.get(PredefinedAtom::Prototype)?;

//         let constructor_proxy: Constructor = globals.get(PredefinedAtom::Proxy)?;

//         let function_get_own_property_descriptor: Function =
//             constructor_object.get(PredefinedAtom::GetOwnPropertyDescriptor)?;

//         let constructor_date: Constructor = globals.get(PredefinedAtom::Date)?;
//         let prototype_date: Object = constructor_date.get(PredefinedAtom::Prototype)?;

//         let constructor_map: Constructor = globals.get(PredefinedAtom::Map)?;
//         let prototype_map: Object = constructor_map.get(PredefinedAtom::Prototype)?;

//         let constructor_weak_map: Constructor = globals.get(PredefinedAtom::WeakMap)?;
//         let prototype_weak_map: Object = constructor_map.get(PredefinedAtom::Prototype)?;

//         let constructor_set: Constructor = globals.get(PredefinedAtom::Set)?;
//         let prototype_set: Object = constructor_set.get(PredefinedAtom::Prototype)?;

//         let constructor_regexp: Constructor = globals.get(PredefinedAtom::RegExp)?;
//         let prototype_regexp: Object = constructor_regexp.get(PredefinedAtom::Prototype)?;

//         let constructor_uint8array: Constructor = globals.get(PredefinedAtom::Uint8Array)?;
//         let constructor_arraybuffer: Constructor = globals.get(PredefinedAtom::ArrayBuffer)?;

//         let constructor_error: Constructor = globals.get(PredefinedAtom::Error)?;
//         let constructor_type_error: Constructor = ctx.globals().get(PredefinedAtom::TypeError)?;
//         let constructor_range_error: Constructor = ctx.globals().get(PredefinedAtom::RangeError)?;
//         let prototype_error: Object = constructor_error.get(PredefinedAtom::Prototype)?;

//         let constructor_array: Object = globals.get(PredefinedAtom::Array)?;
//         let function_array_from: Function = constructor_array.get(PredefinedAtom::From)?;

//         let constructor_array_buffer: Object = globals.get(PredefinedAtom::ArrayBuffer)?;
//         let function_array_buffer_is_view: Function = constructor_array_buffer.get("isView")?;

//         let constructor_bool: Constructor = globals.get(PredefinedAtom::Boolean)?;

//         let constructor_number: Constructor = globals.get(PredefinedAtom::Number)?;
//         let function_parse_float: Function = constructor_number.get("parseFloat")?;
//         let function_parse_int: Function = constructor_number.get("parseInt")?;

//         let constructor_string: Constructor = globals.get(PredefinedAtom::String)?;

//         let constructor_symbol: Constructor = globals.get(PredefinedAtom::Symbol)?;
//         let function_symbol_for: Function = constructor_symbol.get(PredefinedAtom::For)?;

//         let symbol_dispose: Symbol<'js> = constructor_symbol.call(("$Dispose",))?;
//         let symbol_async_dispose: Symbol<'js> = constructor_symbol.call(("$AsyncDispose",))?;

//         constructor_symbol.set("dispose", symbol_dispose.clone())?;
//         constructor_symbol.set("asyncDispose", symbol_async_dispose.clone())?;

// let atom_entries = Atom::from_str(ctx.clone(), "entries")?;
// let atom_keys = Atom::from_str(ctx.clone(), "keys")?;

//         Ok(Self {
//             constructor_map,
//             constructor_set,
//             constructor_weak_map,
//             constructor_date,
//             constructor_proxy,
//             constructor_error,
//             constructor_type_error,
//             constructor_range_error,
//             constructor_regexp,
//             constructor_uint8array,
//             constructor_array_buffer: constructor_arraybuffer,
//             constructor_object,
//             constructor_bool,
//             constructor_number,
//             constructor_string,
//             prototype_object,
//             prototype_date,
//             prototype_regexp,
//             prototype_set,
//             prototype_map,
//             prototype_weak_map,
//             prototype_error,
//             function_array_from,
//             function_array_buffer_is_view,
//             function_get_own_property_descriptor,
//             function_parse_float,
//             function_parse_int,
//             function_symbol_for,
//             symbol_dispose,
//             symbol_async_dispose,
//             atom_entries,
//             atom_keys,
//         })
//     }
// }

#[derive(JsLifetime, Trace)]
pub struct BasePrimordials<'js> {
    // // Constructors
    pub constructor_map: Function<'js>,
    pub constructor_weak_map: Function<'js>,
    pub constructor_set: Function<'js>,
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
    pub atom_entries: String<'js>,
    pub atom_keys: String<'js>,
}

impl<'js> BasePrimordials<'js> {
    pub fn new(ctx: &Ctx<'js>) -> Result<BasePrimordials<'js>> {
        let globals = ctx.globals();

        let constructor_map: Function = globals.get(PredefinedAtom::Map)?;
        let constructor_weak_map: Function = globals.get(PredefinedAtom::WeakMap)?;

        let atom_entries = Atom::from_str(ctx.clone(), "entries")?;
        let atom_keys = Atom::from_str(ctx.clone(), "keys")?;

        let constructor_set: Function = globals.get(PredefinedAtom::Set)?;

        Ok(BasePrimordials {
            constructor_map,
            constructor_weak_map,
            constructor_set,
            atom_entries: atom_entries.to_js_string()?,
            atom_keys: atom_keys.to_js_string()?,
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
}
