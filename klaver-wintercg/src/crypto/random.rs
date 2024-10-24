use klaver::throw;
use klaver_shared::TypedArray;
use rand::RngCore;
use rquickjs::Ctx;

pub fn random_uuid() -> rquickjs::Result<String> {
    let id = uuid::Uuid::new_v4();
    Ok(id.hyphenated().to_string())
}

pub fn random_values<'js>(ctx: Ctx<'js>, buffer: TypedArray<'js>) -> rquickjs::Result<()> {
    let Some(mut raw) = buffer.as_raw() else {
        throw!(ctx, "TypedArray is detached")
    };

    rand::thread_rng().fill_bytes(raw.slice_mut());

    Ok(())
}
