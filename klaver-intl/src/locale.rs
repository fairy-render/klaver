use icu::locid::{Locale, locale};
use klaver_base::create_export;
use klaver_util::throw_if;
use rquickjs::{Ctx, FromJs, Object, class::Trace};

pub struct LocalOptions {
    language: Option<String>,
    script: Option<String>,
    region: Option<String>,
    calendar: Option<String>,
}

impl<'js> FromJs<'js> for LocalOptions {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj = Object::from_js(ctx, value)?;

        Ok(LocalOptions {
            language: obj.get("language")?,
            script: obj.get("script")?,
            region: obj.get("region")?,
            calendar: obj.get("calendar")?,
        })
    }
}

#[derive(rquickjs::JsLifetime, Debug, Clone)]
#[rquickjs::class(rename = "Locale")]
pub struct JsLocale {
    locale: Locale,
}

impl<'js> Trace<'js> for JsLocale {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl From<Locale> for JsLocale {
    fn from(value: Locale) -> Self {
        JsLocale { locale: value }
    }
}

impl From<JsLocale> for Locale {
    fn from(value: JsLocale) -> Self {
        value.locale
    }
}

#[rquickjs::methods]
impl<'js> JsLocale {
    #[qjs(constructor)]
    pub fn new(
        ctx: Ctx<'_>,
        locale: String,
        options: Option<LocalOptions>,
    ) -> rquickjs::Result<JsLocale> {
        let locale: Locale = throw_if!(ctx, locale.parse());

        Ok(JsLocale { locale })
    }

    #[qjs(get)]
    pub fn language(&self) -> rquickjs::Result<String> {
        Ok(self.locale.id.language.as_str().to_string())
    }
}

pub fn current_local(ctx: &Ctx<'_>) -> rquickjs::Result<Locale> {
    if let Some(local) = sys_locale::get_locale() {
        let locale: Locale = throw_if!(ctx, local.parse());
        Ok(locale)
    } else {
        Ok(locale!("en_gb"))
    }
}

create_export!(JsLocale);
