use core::{fmt, marker::PhantomData, ptr::NonNull};
use rquickjs::{class::Trace, ArrayBuffer, Ctx, FromJs, IntoJs};

pub enum Buffer<'js> {
    ArrayBuffer(rquickjs::ArrayBuffer<'js>),
    TypedArray(TypedArray<'js>),
}

impl<'js> Buffer<'js> {
    pub fn as_raw(&self) -> Option<RawBuffer<'_>> {
        match self {
            Self::ArrayBuffer(b) => b.as_raw().map(|m| RawBuffer {
                len: m.len,
                ptr: m.ptr,
                life: PhantomData,
            }),
            Self::TypedArray(b) => b.as_raw(),
        }
    }

    pub fn is(ctx: &Ctx<'js>, value: &rquickjs::Value<'js>) -> rquickjs::Result<bool> {
        Ok(Self::from_js(ctx, value.clone()).is_ok())
    }

    pub fn detach(&self) -> rquickjs::Result<()> {
        self.array_buffer()?.detach();
        Ok(())
    }

    pub fn len(&self) -> usize {
        match self {
            Self::ArrayBuffer(b) => b.len(),
            Self::TypedArray(b) => b.len(),
        }
    }

    pub fn array_buffer(&self) -> rquickjs::Result<ArrayBuffer<'js>> {
        match self {
            Self::ArrayBuffer(b) => Ok(b.clone()),
            Self::TypedArray(b) => b.as_array_buffer(),
        }
    }
}

impl<'js> FromJs<'js> for Buffer<'js> {
    fn from_js(ctx: &Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let obj = value
            .try_into_object()
            .map_err(|m| rquickjs::Error::new_from_js(m.type_name(), "object"))?;

        if obj.is_array_buffer() {
            let buffer = rquickjs::ArrayBuffer::from_object(obj)
                .ok_or_else(|| rquickjs::Error::new_from_js("object", "ArrayBuffer"))?;
            Ok(Buffer::ArrayBuffer(buffer))
        } else {
            Ok(Buffer::TypedArray(TypedArray::from_js(
                ctx,
                obj.into_value(),
            )?))
        }
    }
}

impl<'js> IntoJs<'js> for Buffer<'js> {
    fn into_js(self, _ctx: &Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        match self {
            Self::ArrayBuffer(b) => Ok(b.into_value()),
            Self::TypedArray(b) => Ok(b.into_value()),
        }
    }
}

impl<'js> Trace<'js> for Buffer<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            Self::ArrayBuffer(b) => b.trace(tracer),
            Self::TypedArray(b) => b.trace(tracer),
        }
    }
}

impl<'js> fmt::Display for Buffer<'js> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArrayBuffer(b) => write!(f, "ArrayBuffer[len={}]", b.len()),
            Self::TypedArray(b) => b.fmt(f),
        }
    }
}

pub struct RawBuffer<'a> {
    len: usize,
    ptr: NonNull<u8>,
    life: PhantomData<&'a u8>,
}

impl<'a> RawBuffer<'a> {
    pub fn slice_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    pub fn slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

#[derive(Debug)]
pub enum TypedArray<'js> {
    U8(rquickjs::TypedArray<'js, u8>),
    I8(rquickjs::TypedArray<'js, i8>),
    I16(rquickjs::TypedArray<'js, i16>),
    U16(rquickjs::TypedArray<'js, u16>),
    I32(rquickjs::TypedArray<'js, i32>),
    U32(rquickjs::TypedArray<'js, u32>),
}

impl<'js> TypedArray<'js> {
    pub fn as_raw(&self) -> Option<RawBuffer<'_>> {
        let raw = match self {
            Self::I16(m) => m.as_raw(),
            Self::I32(m) => m.as_raw(),
            Self::U16(m) => m.as_raw(),
            Self::U32(m) => m.as_raw(),
            Self::I8(m) => m.as_raw(),
            Self::U8(m) => m.as_raw(),
        };

        raw.map(|m| RawBuffer {
            len: m.len,
            ptr: m.ptr,
            life: PhantomData,
        })
    }

    pub fn len(&self) -> usize {
        match self {
            Self::I16(m) => m.len(),
            Self::I32(m) => m.len(),
            Self::U16(m) => m.len(),
            Self::U32(m) => m.len(),
            Self::I8(m) => m.len(),
            Self::U8(m) => m.len(),
        }
    }

    pub fn as_array_buffer(&self) -> rquickjs::Result<ArrayBuffer<'js>> {
        let raw = match self {
            Self::I16(m) => m.arraybuffer(),
            Self::I32(m) => m.arraybuffer(),
            Self::U16(m) => m.arraybuffer(),
            Self::U32(m) => m.arraybuffer(),
            Self::I8(m) => m.arraybuffer(),
            Self::U8(m) => m.arraybuffer(),
        };

        raw
    }

    pub fn into_value(self) -> rquickjs::Value<'js> {
        match self {
            TypedArray::U8(b) => b.into_value(),
            TypedArray::I8(b) => b.into_value(),
            TypedArray::I16(b) => b.into_value(),
            TypedArray::U16(b) => b.into_value(),
            TypedArray::I32(b) => b.into_value(),
            TypedArray::U32(b) => b.into_value(),
        }
    }
}

impl<'js> FromJs<'js> for TypedArray<'js> {
    fn from_js(
        _ctx: &rquickjs::prelude::Ctx<'js>,
        value: rquickjs::Value<'js>,
    ) -> rquickjs::Result<Self> {
        if let Ok(a) = rquickjs::TypedArray::<u8>::from_value(value.clone()) {
            Ok(TypedArray::U8(a))
        } else if let Ok(a) = rquickjs::TypedArray::<i8>::from_value(value.clone()) {
            Ok(TypedArray::I8(a))
        } else if let Ok(a) = rquickjs::TypedArray::<u16>::from_value(value.clone()) {
            Ok(TypedArray::U16(a))
        } else if let Ok(a) = rquickjs::TypedArray::<i16>::from_value(value.clone()) {
            Ok(TypedArray::I16(a))
        } else if let Ok(a) = rquickjs::TypedArray::<u32>::from_value(value.clone()) {
            Ok(TypedArray::U32(a))
        } else if let Ok(a) = rquickjs::TypedArray::<i32>::from_value(value.clone()) {
            Ok(TypedArray::I32(a))
        } else {
            Err(rquickjs::Error::new_from_js(
                value.type_name(),
                "TypedArray",
            ))
        }
    }
}

impl<'js> IntoJs<'js> for TypedArray<'js> {
    fn into_js(self, _ctx: &Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        Ok(self.into_value())
    }
}

impl<'js> Trace<'js> for TypedArray<'js> {
    fn trace<'a>(&self, tracer: rquickjs::class::Tracer<'a, 'js>) {
        match self {
            TypedArray::U8(b) => b.trace(tracer),
            TypedArray::I8(b) => b.trace(tracer),
            TypedArray::I16(b) => b.trace(tracer),
            TypedArray::U16(b) => b.trace(tracer),
            TypedArray::I32(b) => b.trace(tracer),
            TypedArray::U32(b) => b.trace(tracer),
        }
    }
}

impl<'js> fmt::Display for TypedArray<'js> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I16(m) => write!(f, "I16Array[len={}]", m.len()),
            Self::I32(m) => write!(f, "I32Array[len={}]", m.len()),
            Self::U16(m) => write!(f, "U16Array[len={}]", m.len()),
            Self::U32(m) => write!(f, "U21Array[len={}]", m.len()),
            Self::I8(m) => write!(f, "I8Array[len={}]", m.len()),
            Self::U8(m) => write!(f, "U8Array[len={}]", m.len()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bytes(pub Vec<u8>);

impl<'js> IntoJs<'js> for Bytes {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        ArrayBuffer::new(ctx.clone(), self.0).map(|m| m.into_value())
    }
}
