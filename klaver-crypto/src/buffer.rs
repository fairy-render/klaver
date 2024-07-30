use std::{marker::PhantomData, ptr::NonNull};

use rquickjs::{class::Trace, Ctx, FromJs, IntoJs};

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
