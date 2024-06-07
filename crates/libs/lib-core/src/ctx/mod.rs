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
    span: Option<Context>,
}

// Constructor.
impl Ctx {
    fn new() -> Self {
        // Ctx {user_id: Uuid::nil()}
        Ctx {
            request_id: Uuid::new_v4(),
            span: None,
        }
    }

    pub fn new_with_span(span_cxt: Context) -> Self {
        Ctx {
            request_id: Uuid::new_v4(),
            span: Some(span_cxt),
        }
    }

    pub fn new_with_id(id: Uuid) -> Result<Self> {
        if id == Uuid::nil() {
            Err(Error::CtxCannotNewRootCtx)
        } else {
            Ok(Self { request_id: id, span: None })
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

    pub fn get_request_span(&mut self) -> &Context {
        if self.span.is_some() {
            let ctx = self.span.as_ref();
            ctx.unwrap()
        } else {
            let tracer = get_global_trace("flowlocker".to_string());
            let span = tracer.start("lock_new_process");
            self.span = Some(Context::current_with_span(span));
            self.span.as_ref().unwrap()
        }
    }
}