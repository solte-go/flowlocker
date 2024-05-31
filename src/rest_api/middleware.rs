use std::sync::Arc;
use axum::http::{Method, Uri};
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::{json, to_value};
use tracing::{debug, info};
use super::error::{Error};


pub async fn mw_response_map(
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    debug!("{:<12} - mw_response_map", "RES_MAPPER");
    // let uuid = Uuid::new_v4();

    // let rpc_info = res.extensions().get::<Arc<RpcInfo>>().map(Arc::as_ref);

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

    info!("{:?}, {:?}", uri, req_method);
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
