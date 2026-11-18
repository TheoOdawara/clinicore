use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, Response};
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{DefaultOnFailure, TraceLayer};
use tracing::Span;

const UNLOGGED_PATH: &str = "/health";

type MakeSpan = fn(&Request<Body>) -> Span;
type OnResponse = fn(&Response<Body>, Duration, &Span);

pub fn layer() -> TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    MakeSpan,
    (),
    OnResponse,
    (),
    (),
    DefaultOnFailure,
> {
    TraceLayer::new_for_http()
        .make_span_with(request_span as MakeSpan)
        .on_request(())
        .on_response(response_event as OnResponse)
        .on_body_chunk(())
        .on_eos(())
}

fn request_span(request: &Request<Body>) -> Span {
    if request.uri().path() == UNLOGGED_PATH {
        return Span::none();
    }
    tracing::info_span!("request", method = %request.method(), path = %request.uri().path())
}

fn response_event(response: &Response<Body>, latency: Duration, span: &Span) {
    if span.is_none() {
        return;
    }
    tracing::info!(
        parent: span,
        status = response.status().as_u16(),
        duration_ms = latency.as_millis(),
        "response"
    );
}
