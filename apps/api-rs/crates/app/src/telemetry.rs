use std::time::Duration;

use axum::http::{Method, Request, Response};
use clinicore_core::config::{AppEnv, Config, LogLevel};
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{MakeSpan, OnResponse, TraceLayer};
use tracing::level_filters::LevelFilter;
use tracing::{Span, Subscriber};
use tracing_subscriber::filter::Targets;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;

pub fn subscriber<Writer>(config: &Config, writer: Writer) -> Box<dyn Subscriber + Send + Sync>
where
    Writer: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    let level = match config.log_level {
        LogLevel::Error => LevelFilter::ERROR,
        LogLevel::Warn => LevelFilter::WARN,
        LogLevel::Info => LevelFilter::INFO,
        LogLevel::Debug => LevelFilter::DEBUG,
        LogLevel::Trace => LevelFilter::TRACE,
        LogLevel::Off => LevelFilter::OFF,
    };
    let targets = Targets::new()
        .with_default(level.min(LevelFilter::WARN))
        .with_target("clinicore_app", level)
        .with_target("clinicore_core", level);

    if config.app_env == AppEnv::Development {
        return Box::new(
            tracing_subscriber::fmt()
                .with_max_level(level)
                .with_writer(writer)
                .finish()
                .with(targets),
        );
    }
    Box::new(
        tracing_subscriber::fmt()
            .json()
            .flatten_event(true)
            .with_span_list(false)
            .with_max_level(level)
            .with_writer(writer)
            .finish()
            .with(targets),
    )
}

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

pub fn request_log()
-> TraceLayer<SharedClassifier<ServerErrorsAsFailures>, RequestSpan, (), RequestLine, (), (), ()> {
    TraceLayer::new_for_http()
        .make_span_with(RequestSpan)
        .on_request(())
        .on_response(RequestLine)
        .on_body_chunk(())
        .on_eos(())
        .on_failure(())
}
