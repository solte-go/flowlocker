use axum::body::Body;
use axum::extract::FromRequestParts;
use std::sync::Arc;

use axum::http::request::Parts;
use axum::http::{Method, Request, Uri};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{async_trait, Json};
use http::StatusCode;

use serde::Serialize;
use serde_json::{json, to_value};
use tracing::{debug, info};
use uuid::Uuid;

use super::error::Result;
use super::error::{ApiError, Error};
use lib_core::ctx::Ctx;

#[derive(Debug, Clone)]
pub struct RequestInfo {
    pub id: Option<Uuid>,
    pub method: String,
    pub path: String,
    pub status_code: Option<StatusCode>,
}

#[derive(Clone, Serialize, Debug)]
pub enum RequestInfoError {
    RequestCantProcessParts,
}

pub async fn mw_ctx_resolver(mut req: Request<Body>, next: Next) -> Result<Response> {
    debug!("{:<12} - mw_ctx_resolver", "MIDDLEWARE");

    let request_ctx = _ctx_resolve().await;
    req.extensions_mut().insert(request_ctx.clone());

    let path = req.uri().path().parse().unwrap_or_else(|_| "/".to_string());

    let req_info = RequestInfo {
        path: format!("r{}", path).to_string(),
        method: (&req.method()).to_string(),
        id: Some(request_ctx.unwrap().0.get_request_id()), //TODO FIX ME
        status_code: None,
    };

    req.extensions_mut().insert(Arc::new(req_info));
    let res = next.run(req).await;

    // let req_info = req.extensions().get::<Arc<RequestInfo>>().map(Arc::as_ref);
    // println!("{:?}", req_info);
    // info!("request_id: {:?} path: {:?} status: {:?}",  req_info.id.unwrap_or_default(), req_info.path, res.status());

    // let ctx = res.extensions_mut().get::<CtxExtResult>();
    // let ctx = ctx.unwrap_or_else(|| );

    // info!("request_id: {:?}, status: {:?}",  request_ctx.unwrap().0, resp.status());

    Ok(res)
}

pub async fn log_result(req: Request<Body>, next: Next) -> Result<Response> {
    let req_info = req
        .extensions()
        .get::<Arc<RequestInfo>>()
        .map(Arc::as_ref)
        .ok_or(ApiError::ReqParts(
            RequestInfoError::RequestCantProcessParts,
        ))?;
    let id = req_info.id;

    info!(
        "event: request_started, id: {:?} path: {:?}",
        id, req_info.path
    );

    Ok(next.run(req).await)
}

async fn _ctx_resolve() -> CtxExtResult {
    let ctx = Ctx::default();

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
    let error_response = client_status_error
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

    info!(
        "event: request_completed, id: {:?}, path: {:?}, method: {:?}, status code: {:?}",
        ctx.get_request_id(),
        uri,
        req_method,
        res.status()
    );
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

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for RequestInfo {
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        let p = parts
            .extensions
            .get::<RequestInfo>()
            .ok_or(ApiError::ReqParts(
                RequestInfoError::RequestCantProcessParts,
            ))?
            .clone();
        Ok(p)
    }
}

type CtxExtResult = core::result::Result<CtxW, CtxExtError>;

#[derive(Clone, Serialize, Debug)]
pub enum CtxExtError {
    CtxNotInRequestExt,
    CtxCreateFail(String),
}
