use std::error::Error;

use tracing::level_filters::LevelFilter;

use crate::config::AppEnv;

pub type TelemetryError = Box<dyn Error + Send + Sync + 'static>;

pub fn init(app_env: AppEnv, log_level: LevelFilter) -> Result<(), TelemetryError> {
    let builder = tracing_subscriber::fmt().with_max_level(log_level);
    match app_env {
        AppEnv::Development => builder.try_init(),
        AppEnv::Production | AppEnv::Test => builder.json().try_init(),
    }
}

#[cfg(test)]
mod tests {
    use tracing::Level;

    use super::*;

    #[test]
    fn the_subscriber_filters_below_the_configured_level_and_installs_once() {
        init(AppEnv::Test, LevelFilter::WARN).expect("no subscriber is installed yet");

        assert!(tracing::enabled!(Level::WARN));
        assert!(!tracing::enabled!(Level::INFO));
        assert!(init(AppEnv::Development, LevelFilter::INFO).is_err());
    }
}
