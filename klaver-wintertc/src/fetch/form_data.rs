use std::cell::RefCell;

use klaver_core::{
    Exportable,
    value::{
        Pair,
        iterable::{IterableProtocol, JsNativeIterator, NativeIteratorInterface},
    },
};
use rquickjs::{
    Class, Coerced, Ctx, FromJs, Function, IntoJs, JsLifetime, String, Value,
    class::{JsClass, Trace},
    prelude::Opt,
};

use crate::blob::{Blob, File};

type Entry<'js> = (String<'js>, FormDataValue<'js>);

/// A `FormData` entry's value. Per the "create an entry" algorithm
/// (<https://xhr.spec.whatwg.org/#create-an-entry>), a `Blob` value is always converted to a
/// `File` on the way in - `get`/`getAll` therefore only ever hand back a string or a `File`,
/// never a bare `Blob`.
#[derive(Clone, Trace)]
pub enum FormDataValue<'js> {
    String(String<'js>),
    File(Class<'js, File<'js>>),
}

impl<'js> FormDataValue<'js> {
    fn into_value(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        match self {
            FormDataValue::String(s) => s.into_js(ctx),
            FormDataValue::File(file) => file.into_js(ctx),
        }
    }
}

/// The `value` argument accepted by `FormData.prototype.append`/`.set`: a string, or a `Blob`/
/// `File` (in which case an optional `filename` may follow).
pub enum FormDataEntryValue<'js> {
    String(String<'js>),
    File(Class<'js, File<'js>>),
    Blob(Class<'js, Blob<'js>>),
}

impl<'js> FromJs<'js> for FormDataEntryValue<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        if let Ok(file) = Class::<File<'js>>::from_js(ctx, value.clone()) {
            Ok(FormDataEntryValue::File(file))
        } else if let Ok(blob) = Class::<Blob<'js>>::from_js(ctx, value.clone()) {
            Ok(FormDataEntryValue::Blob(blob))
        } else {
            let Coerced(s) = Coerced::<String<'js>>::from_js(ctx, value)?;
            Ok(FormDataEntryValue::String(s))
        }
    }
}

/// Snapshotting iterator over a `FormData`'s entries, used for `entries()`, `keys()`, `values()`
/// and the `for...of` protocol - mirrors `URLSearchParams`'s `EntriesIter`.
pub struct EntriesIter<'js> {
    items: Vec<Entry<'js>>,
    pos: RefCell<usize>,
}

impl<'js> EntriesIter<'js> {
    fn new(items: Vec<Entry<'js>>) -> Self {
        EntriesIter {
            items,
            pos: RefCell::new(0),
        }
    }
}

impl<'js> Trace<'js> for EntriesIter<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        for (k, v) in &self.items {
            k.trace(tracer);
            v.trace(tracer);
        }
    }
}

impl<'js> NativeIteratorInterface<'js> for EntriesIter<'js> {
    type Item = Pair<String<'js>, Value<'js>>;

    fn next(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Option<Self::Item>> {
        let mut pos = self.pos.borrow_mut();
        let Some((k, v)) = self.items.get(*pos) else {
            return Ok(None);
        };
        let pair = Pair(k.clone(), v.clone().into_value(ctx)?);
        *pos += 1;
        Ok(Some(pair))
    }

    fn returns(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        Ok(())
    }
}

/// Like [`EntriesIter`] but yields just the keys, for `keys()`.
struct KeysIter<'js> {
    items: Vec<String<'js>>,
    pos: RefCell<usize>,
}

impl<'js> Trace<'js> for KeysIter<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        for item in &self.items {
            item.trace(tracer);
        }
    }
}

impl<'js> NativeIteratorInterface<'js> for KeysIter<'js> {
    type Item = String<'js>;

    fn next(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<Option<Self::Item>> {
        let mut pos = self.pos.borrow_mut();
        let Some(item) = self.items.get(*pos) else {
            return Ok(None);
        };
        let item = item.clone();
        *pos += 1;
        Ok(Some(item))
    }

    fn returns(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        Ok(())
    }
}

/// Like [`EntriesIter`] but yields just the values, for `values()`.
struct ValuesIter<'js> {
    items: Vec<FormDataValue<'js>>,
    pos: RefCell<usize>,
}

impl<'js> Trace<'js> for ValuesIter<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        for item in &self.items {
            item.trace(tracer);
        }
    }
}

impl<'js> NativeIteratorInterface<'js> for ValuesIter<'js> {
    type Item = Value<'js>;

    fn next(&self, ctx: &Ctx<'js>) -> rquickjs::Result<Option<Self::Item>> {
        let mut pos = self.pos.borrow_mut();
        let Some(item) = self.items.get(*pos).cloned() else {
            return Ok(None);
        };
        *pos += 1;
        Ok(Some(item.into_value(ctx)?))
    }

    fn returns(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        Ok(())
    }
}

/// Percent-encode the bytes the multipart/form-data encoding algorithm requires escaping in a
/// `name`/`filename` parameter: CR, LF and `"` (notably *not* backslash).
fn escape_field(input: &str) -> std::string::String {
    input.replace('\r', "%0D").replace('\n', "%0A").replace('"', "%22")
}

/// Generates a boundary token unique enough to not collide with the encoded field data. Not
/// cryptographically random - it only needs to not appear in the payload, which is what the
/// per-process, per-call random seed `RandomState` gives us for free without pulling in a `rand`
/// dependency for the non-`crypto` build of the `fetch` feature.
fn generate_boundary() -> std::string::String {
    use std::hash::{BuildHasher, Hasher};

    let a = std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish();
    let b = std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish();

    format!("----KlaverFormBoundary{a:016x}{b:016x}")
}

#[rquickjs::class]
pub struct FormData<'js> {
    entries: RefCell<Vec<Entry<'js>>>,
}

impl<'js> Trace<'js> for FormData<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        for (k, v) in self.entries.borrow().iter() {
            k.trace(tracer);
            v.trace(tracer);
        }
    }
}

unsafe impl<'js> JsLifetime<'js> for FormData<'js> {
    type Changed<'to> = FormData<'to>;
}

impl<'js> FormData<'js> {
    pub fn new_native() -> FormData<'js> {
        FormData {
            entries: RefCell::new(Vec::new()),
        }
    }

    /// Used by `Body::formData()` to populate a fresh instance from a parsed
    /// `application/x-www-form-urlencoded` or `multipart/form-data` body.
    pub fn push_string(&self, ctx: &Ctx<'js>, name: &str, value: &str) -> rquickjs::Result<()> {
        let name = String::from_str(ctx.clone(), name)?;
        let value = String::from_str(ctx.clone(), value)?;
        self.entries
            .borrow_mut()
            .push((name, FormDataValue::String(value)));
        Ok(())
    }

    pub fn push_file(
        &self,
        ctx: &Ctx<'js>,
        name: &str,
        file: Class<'js, File<'js>>,
    ) -> rquickjs::Result<()> {
        let name = String::from_str(ctx.clone(), name)?;
        self.entries
            .borrow_mut()
            .push((name, FormDataValue::File(file)));
        Ok(())
    }

    /// Encodes this `FormData` as a `multipart/form-data` body, per the WHATWG "multipart/form-data
    /// encoding algorithm". Returns the boundary token used (for the `Content-Type` header) along
    /// with the encoded bytes.
    pub fn encode_multipart(&self) -> rquickjs::Result<(std::string::String, Vec<u8>)> {
        let boundary = generate_boundary();
        let mut buf = Vec::new();

        for (name, value) in self.entries.borrow().iter() {
            buf.extend_from_slice(b"--");
            buf.extend_from_slice(boundary.as_bytes());
            buf.extend_from_slice(b"\r\n");
            buf.extend_from_slice(b"Content-Disposition: form-data; name=\"");
            buf.extend_from_slice(escape_field(&name.to_string()?).as_bytes());
            buf.extend_from_slice(b"\"");

            match value {
                FormDataValue::String(s) => {
                    buf.extend_from_slice(b"\r\n\r\n");
                    buf.extend_from_slice(s.to_string()?.as_bytes());
                }
                FormDataValue::File(file) => {
                    buf.extend_from_slice(b"; filename=\"");
                    let file = file.borrow();
                    buf.extend_from_slice(escape_field(&file.name.to_string()?).as_bytes());
                    buf.extend_from_slice(b"\"\r\n");

                    let content_type = match &file.base.ty {
                        Some(ty) if !ty.to_string()?.is_empty() => ty.to_string()?,
                        _ => "application/octet-stream".to_string(),
                    };
                    buf.extend_from_slice(b"Content-Type: ");
                    buf.extend_from_slice(content_type.as_bytes());
                    buf.extend_from_slice(b"\r\n\r\n");

                    if let Some(bytes) = file.base.buffer.as_bytes() {
                        buf.extend_from_slice(bytes);
                    }
                }
            }

            buf.extend_from_slice(b"\r\n");
        }

        buf.extend_from_slice(b"--");
        buf.extend_from_slice(boundary.as_bytes());
        buf.extend_from_slice(b"--");

        Ok((boundary, buf))
    }
}

#[rquickjs::methods]
impl<'js> FormData<'js> {
    #[qjs(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> FormData<'js> {
        FormData::new_native()
    }

    pub fn append(
        &self,
        ctx: Ctx<'js>,
        name: String<'js>,
        value: FormDataEntryValue<'js>,
        filename: Opt<String<'js>>,
    ) -> rquickjs::Result<()> {
        let value = resolve_value(&ctx, value, filename.0)?;
        self.entries.borrow_mut().push((name, value));
        Ok(())
    }

    pub fn set(
        &self,
        ctx: Ctx<'js>,
        name: String<'js>,
        value: FormDataEntryValue<'js>,
        filename: Opt<String<'js>>,
    ) -> rquickjs::Result<()> {
        let value = resolve_value(&ctx, value, filename.0)?;
        let name_str = name.to_string()?;
        let mut found = false;

        let mut entries = self.entries.borrow_mut();
        let mut i = 0;
        while i < entries.len() {
            if entries[i].0.to_string()? == name_str {
                if !found {
                    entries[i].1 = value.clone();
                    found = true;
                    i += 1;
                } else {
                    entries.remove(i);
                }
            } else {
                i += 1;
            }
        }

        if !found {
            entries.push((name, value));
        }

        Ok(())
    }

    pub fn get(&self, ctx: Ctx<'js>, name: String<'js>) -> rquickjs::Result<Option<Value<'js>>> {
        let name = name.to_string()?;
        for (k, v) in self.entries.borrow().iter() {
            if k.to_string()? == name {
                return Ok(Some(v.clone().into_value(&ctx)?));
            }
        }
        Ok(None)
    }

    #[qjs(rename = "getAll")]
    pub fn get_all(&self, ctx: Ctx<'js>, name: String<'js>) -> rquickjs::Result<Vec<Value<'js>>> {
        let name = name.to_string()?;
        let mut out = Vec::new();
        for (k, v) in self.entries.borrow().iter() {
            if k.to_string()? == name {
                out.push(v.clone().into_value(&ctx)?);
            }
        }
        Ok(out)
    }

    pub fn has(&self, name: String<'js>) -> rquickjs::Result<bool> {
        let name = name.to_string()?;
        for (k, _) in self.entries.borrow().iter() {
            if k.to_string()? == name {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn delete(&self, name: String<'js>) -> rquickjs::Result<()> {
        let name = name.to_string()?;
        let mut entries = self.entries.borrow_mut();
        let mut i = 0;
        while i < entries.len() {
            if entries[i].0.to_string()? == name {
                entries.remove(i);
            } else {
                i += 1;
            }
        }
        Ok(())
    }

    pub fn entries(&self) -> JsNativeIterator<'js> {
        JsNativeIterator::new(EntriesIter::new(self.entries.borrow().clone()))
    }

    pub fn keys(&self) -> JsNativeIterator<'js> {
        let items = self.entries.borrow().iter().map(|(k, _)| k.clone()).collect();
        JsNativeIterator::new(KeysIter {
            items,
            pos: RefCell::new(0),
        })
    }

    pub fn values(&self) -> JsNativeIterator<'js> {
        let items = self.entries.borrow().iter().map(|(_, v)| v.clone()).collect();
        JsNativeIterator::new(ValuesIter {
            items,
            pos: RefCell::new(0),
        })
    }

    #[qjs(rename = "forEach")]
    pub fn for_each(&self, ctx: Ctx<'js>, func: Function<'js>) -> rquickjs::Result<()> {
        let items = self.entries.borrow().clone();

        for (k, v) in items {
            let v = v.into_value(&ctx)?;
            func.call::<_, ()>((v, k))?;
        }

        Ok(())
    }
}

impl<'js> IterableProtocol<'js> for FormData<'js> {
    type Iterator = EntriesIter<'js>;

    fn create_iterator(&self, _ctx: &Ctx<'js>) -> rquickjs::Result<Self::Iterator> {
        Ok(EntriesIter::new(self.entries.borrow().clone()))
    }
}

/// Implements the "create an entry" algorithm's value-conversion step
/// (<https://xhr.spec.whatwg.org/#create-an-entry>): a string stays a string; a `Blob` or `File`
/// is (re)wrapped as a `File`, using `filename` if given, else the source `File`'s own name, else
/// `"blob"`.
fn resolve_value<'js>(
    ctx: &Ctx<'js>,
    value: FormDataEntryValue<'js>,
    filename: Option<String<'js>>,
) -> rquickjs::Result<FormDataValue<'js>> {
    Ok(match value {
        FormDataEntryValue::String(s) => FormDataValue::String(s),
        FormDataEntryValue::File(file) => match filename {
            None => FormDataValue::File(file),
            Some(filename) => {
                let source = file.borrow();
                let new_file = File::new_native(
                    source.base.buffer.clone(),
                    source.base.ty.clone(),
                    filename,
                    None,
                );
                drop(source);
                FormDataValue::File(Class::instance(ctx.clone(), new_file)?)
            }
        },
        FormDataEntryValue::Blob(blob) => {
            let filename = match filename {
                Some(filename) => filename,
                None => String::from_str(ctx.clone(), "blob")?,
            };
            let source = blob.borrow();
            let new_file =
                File::new_native(source.buffer.clone(), source.ty.clone(), filename, None);
            drop(source);
            FormDataValue::File(Class::instance(ctx.clone(), new_file)?)
        }
    })
}

impl<'js> Exportable<'js> for FormData<'js> {
    fn export<T>(
        ctx: &Ctx<'js>,
        _registry: &klaver_core::Registry,
        target: &T,
    ) -> rquickjs::Result<()>
    where
        T: klaver_core::ExportTarget<'js>,
    {
        target.set(
            ctx,
            FormData::NAME,
            Class::<Self>::create_constructor(ctx)?,
        )?;

        Self::add_iterable_prototype(ctx)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use klaver_core::Subclass;
    use rquickjs::{CatchResultExt, Context, Function, Runtime};

    use crate::blob::{Blob, File, NativeBlob};

    /// Runs `body` as the contents of a plain function, with global `FormData`/`Blob`/`File`
    /// constructors available. `body` is expected to throw on failure (e.g. via a plain
    /// `if (...) throw ...`).
    fn run(body: &str) {
        let runtime = Runtime::new().unwrap();
        let context = Context::full(&runtime).unwrap();

        context
            .with(|ctx| {
                ctx.globals().set(
                    FormData::NAME,
                    Class::<FormData>::create_constructor(&ctx)?,
                )?;
                FormData::add_iterable_prototype(&ctx)?;
                ctx.globals()
                    .set("Blob", Class::<Blob>::create_constructor(&ctx)?)?;
                Blob::add_blob_prototype(&ctx)?;
                ctx.globals()
                    .set("File", Class::<File>::create_constructor(&ctx)?)?;
                File::inherit(&ctx)?;

                let test_fn: Function = ctx.eval(format!("(() => {{\n{body}\n}})"))?;

                if let Err(err) = test_fn.call::<_, ()>(()).catch(&ctx) {
                    panic!("{err}");
                }

                rquickjs::Result::Ok(())
            })
            .unwrap();
    }

    #[test]
    fn append_and_get_string() {
        run(r#"
            const fd = new FormData();
            fd.append("a", "1");
            fd.append("b", 2);
            if (fd.get("a") !== "1") throw new Error(`a was ${fd.get("a")}`);
            if (fd.get("b") !== "2") throw new Error(`b was ${fd.get("b")}`);
            if (fd.get("missing") !== undefined) throw new Error("expected undefined");
        "#);
    }

    #[test]
    fn appending_a_blob_wraps_it_as_a_file_named_blob() {
        run(r#"
            const fd = new FormData();
            fd.append("file", new Blob(["hi"]));
            const value = fd.get("file");
            if (!(value instanceof File)) throw new Error("expected a File instance");
            if (!(value instanceof Blob)) throw new Error("expected File to be a Blob");
            if (value.name !== "blob") throw new Error(`name was ${value.name}`);
            if (value.size !== 2) throw new Error(`size was ${value.size}`);
        "#);
    }

    #[test]
    fn appending_a_blob_with_a_filename_uses_it_as_the_name() {
        run(r#"
            const fd = new FormData();
            fd.append("file", new Blob(["hi"], { type: "text/plain" }), "hi.txt");
            const value = fd.get("file");
            if (!(value instanceof File)) throw new Error("expected a File instance");
            if (value.name !== "hi.txt") throw new Error(`name was ${value.name}`);
            if (value.type !== "text/plain") throw new Error(`type was ${value.type}`);
        "#);
    }

    #[test]
    fn appending_a_file_preserves_its_own_name_when_no_filename_is_given() {
        run(r#"
            const fd = new FormData();
            fd.append("file", new File(["hi"], "original.txt"));
            const value = fd.get("file");
            if (value.name !== "original.txt") throw new Error(`name was ${value.name}`);
        "#);
    }

    #[test]
    fn appending_a_file_with_a_filename_overrides_its_name() {
        run(r#"
            const fd = new FormData();
            fd.append("file", new File(["hi"], "original.txt"), "renamed.txt");
            const value = fd.get("file");
            if (value.name !== "renamed.txt") throw new Error(`name was ${value.name}`);
        "#);
    }

    #[test]
    fn get_all_returns_every_matching_value() {
        run(r#"
            const fd = new FormData();
            fd.append("a", "1");
            fd.append("a", "2");
            const all = fd.getAll("a");
            if (all.length !== 2 || all[0] !== "1" || all[1] !== "2") {
                throw new Error(`getAll(a) was ${all}`);
            }
        "#);
    }

    #[test]
    fn has_and_delete() {
        run(r#"
            const fd = new FormData();
            fd.append("a", "1");
            if (!fd.has("a")) throw new Error("expected has(a)");
            fd.delete("a");
            if (fd.has("a")) throw new Error("did not expect has(a) after delete");
        "#);
    }

    #[test]
    fn set_collapses_duplicates_keeping_first_position() {
        run(r#"
            const fd = new FormData();
            fd.append("a", "1");
            fd.append("b", "2");
            fd.append("a", "3");
            fd.set("a", "new");

            const entries = [...fd.entries()].map(([k, v]) => `${k}=${v}`);
            if (entries.join(",") !== "a=new,b=2") throw new Error(`entries were ${entries}`);
        "#);
    }

    #[test]
    fn iteration_and_for_each() {
        run(r#"
            const fd = new FormData();
            fd.append("a", "1");
            fd.append("b", "2");

            const keys = [...fd.keys()];
            if (keys.join(",") !== "a,b") throw new Error(`keys were ${keys}`);

            const values = [...fd.values()];
            if (values.join(",") !== "1,2") throw new Error(`values were ${values}`);

            const seen = [];
            fd.forEach((value, key) => seen.push(`${key}=${value}`));
            if (seen.join(",") !== "a=1,b=2") throw new Error(`seen was ${seen}`);

            const spread = [...fd].map(([k, v]) => `${k}=${v}`);
            if (spread.join(",") !== "a=1,b=2") throw new Error(`spread was ${spread}`);
        "#);
    }
}
