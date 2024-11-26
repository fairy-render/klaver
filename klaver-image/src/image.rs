use std::io::Cursor;

use image::{imageops::FilterType, ImageReader};
use rquickjs::{function::Opt, ArrayBuffer, Ctx, FromJs, Object};
use rquickjs_util::buffer::Buffer;
use rquickjs_util::{throw, throw_if, StringRef};

pub struct ImageFormat(image::ImageFormat);

impl<'js> FromJs<'js> for ImageFormat {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let format = String::from_js(ctx, value)?;

        let Some(format) = image::ImageFormat::from_extension(&format) else {
            throw!(ctx, format!("Unknown format: '{format}'"))
        };

        Ok(ImageFormat(format))
    }
}

impl ImageFormat {
    pub fn write_to<T: std::io::Write>(
        &self,
        ctx: &Ctx<'_>,
        image: &image::DynamicImage,
        buffer: &mut T,
    ) -> rquickjs::Result<()> {
        match self.0 {
            image::ImageFormat::Png => {
                let encoder = image::codecs::png::PngEncoder::new(buffer);
                throw_if!(ctx, image.write_with_encoder(encoder));
            }
            image::ImageFormat::Jpeg => {
                let encoder = image::codecs::jpeg::JpegEncoder::new(buffer);
                throw_if!(ctx, image.write_with_encoder(encoder));
            }
            image::ImageFormat::Pnm => {
                let encoder = image::codecs::pnm::PnmEncoder::new(buffer);
                throw_if!(ctx, image.write_with_encoder(encoder))
            }
            #[cfg(feature = "webp")]
            image::ImageFormat::WebP => {
                let encoder = throw_if!(ctx, webp::Encoder::from_image(image));
                let img = match encoder.encode_simple(false, 80.0) {
                    Ok(img) => img,
                    Err(_) => throw!(ctx, "Encoding error"),
                };
                throw_if!(ctx, buffer.write_all(&*img));
            }
            _ => {
                throw!(ctx, "Invalid format")
            }
        };

        Ok(())
    }
}
pub struct ImageFilter(FilterType);

impl<'js> FromJs<'js> for ImageFilter {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let m = StringRef::from_js(ctx, value)?;

        let ty = match m.as_str() {
            "nearest" => FilterType::Nearest,
            "triangle" => FilterType::Triangle,
            "catmullrom" => FilterType::CatmullRom,
            "gaussian" => FilterType::Gaussian,
            "lanczos3" => FilterType::Lanczos3,
            _ => return Err(rquickjs::Error::new_from_js("string", "filter type")),
        };

        Ok(ImageFilter(ty))
    }
}

pub struct ReizeOptions {
    width: u32,
    height: u32,
    exact: Option<bool>,
    kind: Option<ImageFilter>,
}

impl<'js> FromJs<'js> for ReizeOptions {
    fn from_js(_ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj: Object<'js> = value.get()?;

        Ok(ReizeOptions {
            width: obj.get("width")?,
            height: obj.get("height")?,
            exact: obj.get("exact")?,
            kind: obj.get("kind")?,
        })
    }
}

#[derive(rquickjs::JsLifetime)]
#[rquickjs::class(rename = "Image")]
pub struct JsImage {
    image: image::DynamicImage,
}

#[rquickjs::methods]
impl JsImage {
    #[qjs(static)]
    pub async fn open(ctx: Ctx<'_>, path: StringRef<'_>) -> rquickjs::Result<JsImage> {
        let content = throw_if!(ctx, tokio::fs::read(path.as_str()).await);

        let reader = throw_if!(
            ctx,
            ImageReader::new(Cursor::new(content)).with_guessed_format()
        );

        let img = throw_if!(ctx, reader.decode());

        Ok(JsImage { image: img })
    }

    #[qjs(constructor)]
    pub fn new<'js>(ctx: Ctx<'js>, buf: Buffer<'js>) -> rquickjs::Result<JsImage> {
        let Some(buffer) = buf.as_raw() else {
            throw!(ctx, "Buffer detached");
        };

        let reader = throw_if!(
            ctx,
            ImageReader::new(Cursor::new(buffer.slice())).with_guessed_format()
        );

        let image = throw_if!(ctx, reader.decode());

        Ok(JsImage { image })
    }

    pub async fn save(
        &self,
        ctx: Ctx<'_>,
        path: StringRef<'_>,
        Opt(fmt): Opt<ImageFormat>,
    ) -> rquickjs::Result<()> {
        let fmt = if let Some(format) = fmt {
            format
        } else {
            ImageFormat(throw_if!(ctx, image::ImageFormat::from_path(path.as_str())))
        };

        let mut buffer = Vec::default();
        fmt.write_to(&ctx, &self.image, &mut buffer)?;

        throw_if!(ctx, tokio::fs::write(path.as_str(), buffer).await);

        Ok(())
    }

    #[qjs(rename = "arrayBuffer")]
    pub async fn array_buffer<'js>(
        &self,
        ctx: Ctx<'js>,
        fmt: ImageFormat,
    ) -> rquickjs::Result<rquickjs::ArrayBuffer<'js>> {
        let mut buffer = Vec::default();
        fmt.write_to(&ctx, &self.image, &mut buffer)?;
        ArrayBuffer::new(ctx, buffer)
    }

    pub fn resize(&self, opts: ReizeOptions) -> rquickjs::Result<JsImage> {
        let exact = opts.exact.unwrap_or_default();
        let ty = opts
            .kind
            .map(|m| m.0)
            .unwrap_or_else(|| FilterType::Nearest);

        let image = if exact {
            self.image.resize_exact(opts.width, opts.height, ty)
        } else {
            self.image.resize(opts.width, opts.height, ty)
        };

        Ok(JsImage { image })
    }

    #[qjs(get)]
    pub fn width(&self) -> rquickjs::Result<u32> {
        Ok(self.image.width())
    }

    #[qjs(get)]
    pub fn height(&self) -> rquickjs::Result<u32> {
        Ok(self.image.height())
    }

    pub fn blur(&self, sigma: f32) -> rquickjs::Result<JsImage> {
        Ok(JsImage {
            image: self.image.blur(sigma),
        })
    }

    pub fn gray(&self) -> rquickjs::Result<JsImage> {
        Ok(JsImage {
            image: self.image.grayscale(),
        })
    }

    pub fn crop(&self, x: u32, y: u32, width: u32, height: u32) -> rquickjs::Result<JsImage> {
        Ok(JsImage {
            image: self.image.crop_imm(x, y, width, height),
        })
    }
}

impl<'js> rquickjs::class::Trace<'js> for JsImage {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}
