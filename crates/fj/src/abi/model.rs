use std::{os::raw::c_void, panic::AssertUnwindSafe};

use crate::{
    abi::{Context, ModelMetadata, ShapeResult},
    models::Error,
};

#[repr(C)]
pub struct Model {
    ptr: *mut c_void,
    metadata: unsafe extern "C" fn(*mut c_void) -> ModelMetadata,
    shape: unsafe extern "C" fn(*mut c_void, Context<'_>) -> ShapeResult,
    free: unsafe extern "C" fn(*mut c_void),
}

impl crate::models::Model for Model {
    fn shape(
        &self,
        ctx: &dyn crate::models::Context,
    ) -> Result<crate::Shape, Error> {
        let ctx = Context::from(&ctx);

        let Model { ptr, shape, .. } = *self;

        let result = unsafe { shape(ptr, ctx) };

        match result {
            super::ffi_safe::Result::Ok(shape) => Ok(shape),
            super::ffi_safe::Result::Err(err) => Err(err.into()),
        }
    }

    fn metadata(&self) -> crate::models::ModelMetadata {
        let Model { ptr, metadata, .. } = *self;

        unsafe { metadata(ptr).into() }
    }
}

impl From<Box<dyn crate::models::Model>> for Model {
    fn from(m: Box<dyn crate::models::Model>) -> Self {
        unsafe extern "C" fn metadata(user_data: *mut c_void) -> ModelMetadata {
            let model = &*(user_data as *mut Box<dyn crate::models::Model>);

            match std::panic::catch_unwind(AssertUnwindSafe(|| {
                model.metadata()
            })) {
                Ok(meta) => meta.into(),
                Err(payload) => crate::abi::on_panic(payload),
            }
        }

        unsafe extern "C" fn shape(
            user_data: *mut c_void,
            ctx: Context<'_>,
        ) -> ShapeResult {
            let model = &*(user_data as *mut Box<dyn crate::models::Model>);

            match std::panic::catch_unwind(AssertUnwindSafe(|| {
                model.shape(&ctx)
            })) {
                Ok(Ok(shape)) => ShapeResult::Ok(shape),
                Ok(Err(err)) => ShapeResult::Err(err.into()),
                Err(payload) => crate::abi::on_panic(payload),
            }
        }

        unsafe extern "C" fn free(user_data: *mut c_void) {
            let model = user_data as *mut Box<dyn crate::models::Model>;

            if let Err(e) = std::panic::catch_unwind(AssertUnwindSafe(|| {
                let model = Box::from_raw(model);
                drop(model);
            })) {
                crate::abi::on_panic(e);
            };
        }

        Model {
            ptr: Box::into_raw(Box::new(m)).cast(),
            metadata,
            shape,
            free,
        }
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        let Model { ptr, free, .. } = *self;

        unsafe {
            free(ptr);
        }
    }
}

// Safety: Our Model type is a FFI-safe version of Box<dyn crate::Model>, and
// Box<dyn crate::Model>: Send+Sync.
unsafe impl Send for Model {}
unsafe impl Sync for Model {}
