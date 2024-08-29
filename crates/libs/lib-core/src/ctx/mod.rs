mod error;

use std::sync::Arc;
use opentelemetry::Context;
pub use self::error::{Error, Result};
use uuid::Uuid;
use tracing::span::Span;
use opentelemetry::trace::{TraceContextExt, Tracer};
use super::tracing::get_global_trace;

#[derive(Debug, Clone)]
pub struct Ctx {
    request_id: Uuid,
}

// Constructor.
impl Ctx {
    fn new() -> Self {
        Ctx {
            request_id: Uuid::new_v4(),
        }
    }

    pub fn new_with_id(id: Uuid) -> Result<Self> {
        if id == Uuid::nil() {
            Err(Error::CtxCannotNewRootCtx)
        } else {
            Ok(Self { request_id: id })
        }
    }
}

impl Default for Ctx {
    fn default() -> Self {
        Ctx::new()
    }
}


impl Ctx {
    pub fn get_request_id(&self) -> Uuid {
        self.request_id
    }
}