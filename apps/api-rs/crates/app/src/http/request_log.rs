use std::time::Duration;

use axum::http::{Method, Request, Response};
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{MakeSpan, OnResponse, TraceLayer};
use tracing::Span;

#[derive(Clone)]
pub struct RequestSpan;

impl<Body> MakeSpan<Body> for RequestSpan {
    fn make_span(&mut self, request: &Request<Body>) -> Span {
        let probe = [Method::GET, Method::HEAD].contains(request.method());
        if probe && request.uri().path() == "/health" {
            return Span::none();
        }
        tracing::info_span!("request", method = %request.method(), path = request.uri().path())
    }
}

#[derive(Clone)]
pub struct RequestLine;

impl<Body> OnResponse<Body> for RequestLine {
    fn on_response(self, response: &Response<Body>, latency: Duration, span: &Span) {
        if span.is_disabled() {
            return;
        }
        tracing::info!(
            status = response.status().as_u16(),
            duration_ms = latency.as_secs_f64() * 1000.0,
            "request"
        );
    }
}

pub fn layer()
-> TraceLayer<SharedClassifier<ServerErrorsAsFailures>, RequestSpan, (), RequestLine, (), (), ()> {
    TraceLayer::new_for_http()
        .make_span_with(RequestSpan)
        .on_request(())
        .on_response(RequestLine)
        .on_body_chunk(())
        .on_eos(())
        .on_failure(())
}
