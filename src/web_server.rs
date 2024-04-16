use std::{sync::Arc, time::Duration};

use alpha_sign::{
    text::{ReadText, WriteText},
    Packet,
};
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot::{self, Sender};
use tower::ServiceBuilder;
use tower_http::{
    services::ServeDir,
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit, ServiceBuilderExt,
};

/// State shared between the main application and the HTTP application.
#[derive(Clone)]
pub struct AppState {
    /// Message channel into which commands can be sent.
    command_tx: tokio::sync::mpsc::UnboundedSender<APICommand>,
}

/// all possible responses to an API command.
pub enum APIResponse {
    ReadText(String),
}

/// Enumerates all messages that can be sent from the webserver to the main program.
/// I don't just use sign commands here because the web server will likely be sending more abstract commands (like "set rotation texts") that are not included in the base sign protocol and handled instead in software.
pub enum APICommand {
    WriteText(WriteText),
    ReadText(ReadText, Sender<APIResponse>),
}

impl AppState {
    /// Creates a new [`AppState`].
    ///
    /// # Arguments
    /// * `command_tx`: Channel into which commands can be sent.
    ///
    /// # Returns
    /// A new [`AppState`].
    pub fn new(command_tx: tokio::sync::mpsc::UnboundedSender<APICommand>) -> Self {
        Self { command_tx }
    }
}

/// Creates a new app for handling HTTP requests.
///
/// # Arguments
/// * `state`: Shared application state.
///
/// # Returns
/// A [`Router`] for handling requests.
pub fn app(state: AppState) -> Router {
    let sensitive_headers: Arc<[_]> = vec![header::AUTHORIZATION, header::COOKIE].into();
    let middleware = ServiceBuilder::new()
        // Mark the `Authorization` and `Cookie` headers as sensitive so it doesn't show in logs
        .sensitive_request_headers(sensitive_headers.clone())
        // Add high level tracing/logging to all requests
        .layer(
            TraceLayer::new_for_http()
                .on_body_chunk(|chunk: &Bytes, latency: Duration, _: &tracing::Span| {
                    tracing::trace!(size_bytes = chunk.len(), latency = ?latency, "sending body chunk")
                })
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(DefaultOnResponse::new().include_headers(true).latency_unit(LatencyUnit::Micros)),
        )
        .sensitive_response_headers(sensitive_headers)
        // Set a timeout
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        // Box the response body so it implements `Default` which is required by axum
        .map_response_body(axum::body::boxed)
        // Compress responses
        .compression()
        // Set a `Content-Type` if there isn't one already.
        .insert_response_header_if_not_present(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );

    Router::new()
        //.route("/script", post(post_script_handler))
        .route("/text/:textKey", put(put_text_handler))
        .route("/text/get/:label", get(get_text_handler))
        .layer(middleware)
        .with_state(state)
        .fallback_service(ServeDir::new("static"))
}

/// Parameters for a PUT to `/text/:textKey`.
#[derive(Debug, Serialize, Deserialize)]
pub struct PutTextParams {
    /// The key to PUT text to.
    #[serde(rename = "textKey")]
    pub text_key: String,
}

/// Body for a PUT to `/text/:textKey`.
#[derive(Debug, Serialize, Deserialize)]
pub struct PutTextRequest {
    /// Text to display.
    pub text: String,
}

/// Handles a PUT to `/text/:textKey`.
///
/// # Arguments
/// * `state`: Shared application state.
/// * `text_key`: Key to write to.
/// * `body`: Request body.
///
/// # Returns
/// JSON with that text returned from the sign
#[axum::debug_handler]
async fn put_text_handler(
    state: State<AppState>,
    Path(PutTextParams { text_key }): Path<PutTextParams>,
    Json(body): Json<PutTextRequest>,
) -> impl IntoResponse {
    // TODO: We should have a list of keys that isn't hard-coded.
    if ["test", "lulzbot", "anycubic"].contains(&text_key.as_str()) {
        state
            .command_tx
            .send(APICommand::WriteText(WriteText::new('A', body.text)))
            .ok(); // TODO: Handle errors

        StatusCode::OK
    } else {
        StatusCode::FORBIDDEN
    }
}

#[derive(Serialize)]
struct GetTextResponse {
    text: String,
}

/// Parameters for a GET to `/text/get`.
#[derive(Debug, Serialize, Deserialize)]
pub struct GetTextParams {
    /// The key to PUT text to.
    pub label: char,
}

#[axum::debug_handler]
async fn get_text_handler(
    state: State<AppState>,
    Path(GetTextParams { label }): Path<GetTextParams>,
) -> impl IntoResponse {
    let (tx, rx) = oneshot::channel::<APIResponse>();
    state
        .command_tx
        .send(APICommand::ReadText(ReadText::new(label), tx))
        .ok(); // TODO handle errors

    match rx.await {
        Ok(APIResponse::ReadText(t)) => Json(GetTextResponse { text: t }).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
