use std::{collections::HashMap, sync::Arc, time::Duration};

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::{
    services::ServeDir,
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit, ServiceBuilderExt,
};

use crate::{AppState, TopicId};

/// Enumerates all messages that can be sent from the webserver to the main program.
pub enum APIEvent {
    /// The contents of the topics was updated.
    TopicsUpdated,
    /// Jump to the specified topic.
    JumpToTopic(TopicId),
    /// The specified topic has been removed.
    TopicDeleted(TopicId),
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
        .route("/help", get(get_help_handler))
        .route("/topics", get(get_topics_handler))
        .route(
            "/topics/:topic",
            get(get_topic_handler)
                .put(put_topic_handler)
                .delete(delete_topic_handler),
        )
        .layer(middleware)
        .with_state(state)
        .fallback_service(ServeDir::new("static"))
}

#[axum::debug_handler]
async fn get_help_handler() -> impl IntoResponse {
    axum::response::Html::from(include_str!("./help.html"))
}

#[derive(Debug, Serialize)]
pub struct GetTopicsResponse {
    topics: HashMap<TopicId, Vec<String>>,
}

async fn get_topics_handler(mut state: State<AppState>) -> impl IntoResponse {
    serde_json::to_string(&GetTopicsResponse {
        topics: state.get_all_topics().await,
    })
    .unwrap()
}

#[derive(Debug, Deserialize)]
pub struct GetTopicRequest {
    pub topic: String,
}

#[derive(Debug, Serialize)]
pub struct GetTopicResponse {
    /// The key to PUT text to.
    pub lines: Vec<String>,
}

async fn get_topic_handler(
    state: State<AppState>,
    Path(GetTopicRequest { topic }): Path<GetTopicRequest>,
) -> impl IntoResponse {
    match state.get_topic(&TopicId(topic)).await {
        Some(lines) => (
            StatusCode::OK,
            serde_json::to_string(&GetTopicResponse { lines }).unwrap(),
        ),
        None => (StatusCode::NOT_FOUND, String::new()),
    }
}

/// Parameters for a PUT to `/topics/:topic`.
#[derive(Debug, Deserialize)]
pub struct PutTopicParams {
    /// The key to PUT text to.
    pub topic: String,
}

/// Body for a PUT to `/text/:topic`.
#[derive(Debug, Deserialize)]
pub struct PutTextRequest {
    /// Text to display.
    /// Lines are limited to 60 characters.
    pub lines: Vec<String>,
}

/// Handles a PUT to `/topics/:topic`.
///
/// # Arguments
/// * `state`: Shared application state.
/// * `text_key`: Key to write to.
/// * `body`: Request body.
///
/// # Returns
/// A status code.
#[axum::debug_handler]
async fn put_topic_handler(
    mut state: State<AppState>,
    Path(PutTopicParams { topic }): Path<PutTopicParams>,
    Json(body): Json<PutTextRequest>,
) -> impl IntoResponse {
    // Reserved for system-level topics.
    if topic.starts_with("__") {
        return StatusCode::BAD_REQUEST;
    }

    let topic_id = TopicId(topic);
    state.set_topic(&topic_id, body.lines).await;
    state.event_tx.send(APIEvent::TopicsUpdated).unwrap();
    state
        .event_tx
        .send(APIEvent::JumpToTopic(topic_id))
        .unwrap();

    StatusCode::OK
}

#[axum::debug_handler]
async fn delete_topic_handler(
    mut state: State<AppState>,
    Path(PutTopicParams { topic }): Path<PutTopicParams>,
) -> impl IntoResponse {
    let topic_id = TopicId(topic);
    state.delete_topic(&topic_id).await;
    state.event_tx.send(APIEvent::TopicsUpdated).unwrap();
    state
        .event_tx
        .send(APIEvent::TopicDeleted(topic_id))
        .unwrap();

    StatusCode::OK
}
