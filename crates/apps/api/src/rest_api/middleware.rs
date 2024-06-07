use std::sync::Arc;
use axum::body::Body;
use axum::extract::{FromRequestParts, State};

use axum::http::{Method, Request, Uri};
use axum::http::request::Parts;
use axum::{async_trait, Json};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use opentelemetry::Context;

use opentelemetry::trace::{TraceContextExt, Tracer};
// use opentelemetry_sdk::trace::Tracer;
use serde::Serialize;
use serde_json::{json, to_value};
use tracing::{debug, info, Instrument};


use super::error::{ApiError, Error};
use super::error::{Result};
use lib_core::ctx::Ctx;
use lib_core::tracing::get_global_trace;

pub async fn mw_ctx_resolver(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response> {
    debug!("{:<12} - mw_ctx_resolver", "MIDDLEWARE");

    let request_ctx = _ctx_resolve(req.uri_mut().to_string()).await;
    req.extensions_mut().insert(request_ctx);

    Ok(next.run(req).await)
}

async fn _ctx_resolve(path: String) -> CtxExtResult {
    // -- Create CtxResult

    let tracer = get_global_trace("flowlocker".to_string());
    let span = tracer.start(path);
    let cx = Context::current_with_span(span);

    let ctx = Ctx::new_with_span(cx);
    info!("New id: {:?}", &ctx.get_request_id());
    Ok(CtxW(ctx))
}


pub async fn mw_response_map(
    ctx: Option<CtxW>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    debug!("{:<12} - mw_response_map", "RES_MAPPER");

    let ctx = ctx.map(|ctx| ctx.0).unwrap_or_default();
    // let rpc_info = res.extensions().get::<Arc<RequestEndpoint>>().map(Arc::as_ref);

    // -- Get the eventual response error.
    let web_error = res.extensions().get::<Arc<Error>>().map(Arc::as_ref);
    let client_status_error = web_error.map(|se| se.client_status_and_error());

    // -- If client error, build the new response.
    let error_response =
        client_status_error
            .as_ref()
            .map(|(status_code, client_error)| {
                let client_error = to_value(client_error).ok();
                let message = client_error.as_ref().and_then(|v| v.get("message"));
                let detail = client_error.as_ref().and_then(|v| v.get("detail"));

                let client_error_body = json!({
					// "id": rpc_info.as_ref().map(|rpc| rpc.id.clone()),
					"error": {
						"message": message,
						"data": {
							// "req_uuid": uuid.to_string(),
							"detail": detail
						}
					}
				});

                debug!("CLIENT ERROR BODY: {client_error_body}");

                // Build the new response from the client_error_body
                (*status_code, Json(client_error_body)).into_response()
            });

    // -- Build and log the server log line.
    let _client_error = client_status_error.unzip().1;
    // TODO: Need to handler if log_request fail (but should not fail request)

    info!("request_id: {:?}, Path: {:?}, Method: {:?}", ctx.get_request_id(), uri, req_method);
    // let _ = log_request(
    //     uuid,
    //     req_method,
    //     uri,
    //     rpc_info,
    //     ctx,
    //     web_error,
    //     client_error)
    //     .await;
    //
    // // info!("\n");

    error_response.unwrap_or(res)
}

#[derive(Debug, Clone)]
pub struct CtxW(pub Ctx);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for CtxW {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        debug!("{:<12} - ctx", "EXTRACTOR");

        parts
            .extensions
            .get::<CtxExtResult>()
            .ok_or(ApiError::CtxExt(CtxExtError::CtxNotInRequestExt))?
            .clone()
            .map_err(ApiError::CtxExt)
    }
}


type CtxExtResult = core::result::Result<CtxW, CtxExtError>;

#[derive(Clone, Serialize, Debug)]
pub enum CtxExtError {
    CtxNotInRequestExt,
    CtxCreateFail(String),
}