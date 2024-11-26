use rquickjs::{class::Trace, Ctx};
use rquickjs_util::{throw_if, StringRef, Val};

#[derive(rquickjs::JsLifetime)]
#[rquickjs::class]
pub struct Handlebars {
    i: handlebars::Handlebars<'static>,
}

impl<'js> Trace<'js> for Handlebars {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl Handlebars {
    #[qjs(constructor)]
    pub fn new() -> Handlebars {
        Handlebars {
            i: handlebars::Handlebars::new(),
        }
    }

    pub fn render(
        &self,
        ctx: Ctx<'_>,
        name: StringRef<'_>,
        context: Val,
    ) -> rquickjs::Result<String> {
        let output = throw_if!(ctx, self.i.render(name.as_str(), &context.0));
        Ok(output)
    }

    #[qjs(rename = "registerTemplate")]
    pub fn register_template(
        &mut self,
        ctx: Ctx<'_>,
        name: String,
        template: String,
    ) -> rquickjs::Result<()> {
        throw_if!(ctx, self.i.register_template_string(&name, template));
        Ok(())
    }
}
