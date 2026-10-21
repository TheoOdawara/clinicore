use tracing::Subscriber;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;

use crate::config::AppEnv;

pub fn subscriber<Writer>(
    app_env: AppEnv,
    level: LevelFilter,
    writer: Writer,
) -> Box<dyn Subscriber + Send + Sync>
where
    Writer: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    let targets = Targets::new()
        .with_default(level.min(LevelFilter::WARN))
        .with_target("clinicore_app", level)
        .with_target("clinicore_core", level)
        .with_target("clinicore_worker", level);

    if app_env == AppEnv::Development {
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
