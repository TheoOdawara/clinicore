use std::fmt;

use tracing::level_filters::LevelFilter;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppEnv {
    Development,
    Production,
    Test,
}

pub struct ApiConfig {
    pub app_env: AppEnv,
    pub log_level: LevelFilter,
    pub port: u16,
    pub database_url: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidEnvironment {
    rejected: Vec<(&'static str, &'static str)>,
}

impl fmt::Display for InvalidEnvironment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Invalid environment:")?;
        for (name, expected) in &self.rejected {
            write!(formatter, "\n  {name}: {expected}")?;
        }
        Ok(())
    }
}

impl std::error::Error for InvalidEnvironment {}

const APP_ENV: &str = "APP_ENV";
const DATABASE_URL: &str = "DATABASE_URL";
const LOG_LEVEL: &str = "LOG_LEVEL";
const PORT: &str = "PORT";

impl ApiConfig {
    pub fn from_env() -> Result<Self, InvalidEnvironment> {
        Self::from_source(|name| std::env::var(name).ok())
    }

    pub fn from_source(read: impl Fn(&str) -> Option<String>) -> Result<Self, InvalidEnvironment> {
        let mut rejected = Vec::new();

        let app_env = read(APP_ENV).and_then(|value| match value.as_str() {
            "development" => Some(AppEnv::Development),
            "production" => Some(AppEnv::Production),
            "test" => Some(AppEnv::Test),
            _ => None,
        });
        if app_env.is_none() {
            rejected.push((APP_ENV, "expected one of development, production, test"));
        }

        let log_level = read(LOG_LEVEL).and_then(|value| match value.as_str() {
            "error" => Some(LevelFilter::ERROR),
            "warn" => Some(LevelFilter::WARN),
            "info" => Some(LevelFilter::INFO),
            "debug" => Some(LevelFilter::DEBUG),
            "trace" => Some(LevelFilter::TRACE),
            "off" => Some(LevelFilter::OFF),
            _ => None,
        });
        if log_level.is_none() {
            rejected.push((
                LOG_LEVEL,
                "expected one of error, warn, info, debug, trace, off",
            ));
        }

        let port = read(PORT)
            .and_then(|value| value.parse::<u16>().ok())
            .filter(|port| *port != 0);
        if port.is_none() {
            rejected.push((PORT, "expected an integer between 1 and 65535"));
        }

        let database_url = read(DATABASE_URL).filter(|value| is_postgres_url(value));
        if database_url.is_none() {
            rejected.push((
                DATABASE_URL,
                "expected a PostgreSQL connection string (postgresql://…)",
            ));
        }

        match (app_env, log_level, port, database_url) {
            (Some(app_env), Some(log_level), Some(port), Some(database_url)) => Ok(Self {
                app_env,
                log_level,
                port,
                database_url,
            }),
            _ => {
                rejected.sort_by_key(|(name, _)| *name);
                Err(InvalidEnvironment { rejected })
            }
        }
    }
}

fn is_postgres_url(value: &str) -> bool {
    ["postgresql://", "postgres://"]
        .iter()
        .any(|scheme| value.len() > scheme.len() && value.starts_with(scheme))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn valid() -> HashMap<&'static str, &'static str> {
        HashMap::from([
            (APP_ENV, "test"),
            (LOG_LEVEL, "warn"),
            (PORT, "3333"),
            (
                DATABASE_URL,
                "postgresql://clinicore:secret-value@localhost:5432/clinicore",
            ),
        ])
    }

    fn load(source: &HashMap<&'static str, &'static str>) -> Result<ApiConfig, InvalidEnvironment> {
        ApiConfig::from_source(|name| source.get(name).map(|value| (*value).to_owned()))
    }

    #[test]
    fn a_complete_environment_is_accepted() {
        let config = load(&valid()).expect("the environment is valid");

        assert_eq!(config.app_env, AppEnv::Test);
        assert_eq!(config.log_level, LevelFilter::WARN);
        assert_eq!(config.port, 3333);
        assert_eq!(
            config.database_url,
            "postgresql://clinicore:secret-value@localhost:5432/clinicore"
        );
    }

    #[test]
    fn each_app_env_and_log_level_is_accepted() {
        for (app_env, expected) in [
            ("development", AppEnv::Development),
            ("production", AppEnv::Production),
            ("test", AppEnv::Test),
        ] {
            let mut source = valid();
            source.insert(APP_ENV, app_env);
            assert_eq!(load(&source).map(|config| config.app_env), Ok(expected));
        }
        for (log_level, expected) in [
            ("error", LevelFilter::ERROR),
            ("warn", LevelFilter::WARN),
            ("info", LevelFilter::INFO),
            ("debug", LevelFilter::DEBUG),
            ("trace", LevelFilter::TRACE),
            ("off", LevelFilter::OFF),
        ] {
            let mut source = valid();
            source.insert(LOG_LEVEL, log_level);
            assert_eq!(load(&source).map(|config| config.log_level), Ok(expected));
        }
    }

    #[test]
    fn an_empty_environment_lists_every_variable_in_alphabetical_order() {
        let error = load(&HashMap::new())
            .err()
            .expect("the environment is empty");

        assert_eq!(
            error.to_string(),
            "Invalid environment:\n  \
             APP_ENV: expected one of development, production, test\n  \
             DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)\n  \
             LOG_LEVEL: expected one of error, warn, info, debug, trace, off\n  \
             PORT: expected an integer between 1 and 65535"
        );
    }

    #[test]
    fn invalid_values_are_refused_without_echoing_them() {
        for (name, value) in [
            (APP_ENV, "staging-secret-value"),
            (LOG_LEVEL, "verbose-secret-value"),
            (PORT, "0"),
            (PORT, "65536"),
            (
                DATABASE_URL,
                "mysql://root:secret-value@localhost/clinicore",
            ),
            (DATABASE_URL, "postgresql://"),
        ] {
            let mut source = valid();
            source.insert(name, value);
            let message = load(&source)
                .err()
                .expect("the value is invalid")
                .to_string();

            assert!(
                message.contains(&format!("\n  {name}: expected")),
                "{name} is listed"
            );
            assert_eq!(message.lines().count(), 2, "only {name} is listed");
            assert!(
                !message.contains("secret-value"),
                "the value of {name} is not echoed"
            );
        }
    }

    #[test]
    fn the_postgres_scheme_alias_is_accepted() {
        let mut source = valid();
        source.insert(DATABASE_URL, "postgres://localhost/clinicore");

        assert!(load(&source).is_ok());
    }
}
