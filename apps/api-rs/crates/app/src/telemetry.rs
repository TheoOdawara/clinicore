use clinicore_core::config::{AppEnv, Config};
use tracing::Subscriber;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;

pub fn subscriber<Writer>(config: &Config, writer: Writer) -> Box<dyn Subscriber + Send + Sync>
where
    Writer: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    let level = config.log_level;
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
