use std::fmt;

use axum_client_ip::ClientIpSource;
use lettre::Address;
use redis::{ConnectionInfo, IntoConnectionInfo};
use sqlx::postgres::PgConnectOptions;
use tracing::level_filters::LevelFilter;
use url::Url;

const ABSOLUTE_URL: &str = "expected an absolute URL with no trailing slash (https://…)";
const NON_EMPTY: &str = "expected a non-empty string";
const PORT_RANGE: &str = "expected an integer between 1 and 65535";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppEnv {
    Development,
    Production,
    Test,
}

#[derive(Clone)]
pub struct Config {
    pub allowed_origins: Vec<String>,
    pub api_url: String,
    pub app_env: AppEnv,
    pub app_origin: String,
    pub client_ip_source: ClientIpSource,
    pub database_url: PgConnectOptions,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub jwt_secret: String,
    pub log_level: LevelFilter,
    pub port: u16,
    pub redis_url: ConnectionInfo,
}

#[derive(Clone)]
pub struct WorkerConfig {
    pub app_env: AppEnv,
    pub database_url: PgConnectOptions,
    pub log_level: LevelFilter,
    pub mail_from: Address,
    pub smtp_host: String,
    pub smtp_password: String,
    pub smtp_port: u16,
    pub smtp_user: String,
}

#[derive(Debug)]
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

struct Reader<Read> {
    read: Read,
    rejected: Vec<(&'static str, &'static str)>,
}

impl<Read: Fn(&str) -> Option<String>> Reader<Read> {
    fn require<Value>(
        &mut self,
        name: &'static str,
        expected: &'static str,
        parse: impl FnOnce(String) -> Option<Value>,
    ) -> Option<Value> {
        let value = (self.read)(name).and_then(parse);
        if value.is_none() {
            self.rejected.push((name, expected));
        }
        value
    }

    fn app_env(&mut self) -> Option<AppEnv> {
        self.require(
            "APP_ENV",
            "expected one of development, production, test",
            |raw| match raw.as_str() {
                "development" => Some(AppEnv::Development),
                "production" => Some(AppEnv::Production),
                "test" => Some(AppEnv::Test),
                _ => None,
            },
        )
    }

    fn database_url(&mut self) -> Option<PgConnectOptions> {
        self.require(
            "DATABASE_URL",
            "expected a PostgreSQL connection string (postgresql://…)",
            |raw| {
                url_with_scheme(raw, &["postgresql", "postgres"])
                    .and_then(|raw| raw.parse::<PgConnectOptions>().ok())
            },
        )
    }

    fn log_level(&mut self) -> Option<LevelFilter> {
        self.require(
            "LOG_LEVEL",
            "expected one of: error, warn, info, debug, trace, off",
            |raw| match raw.as_str() {
                "error" => Some(LevelFilter::ERROR),
                "warn" => Some(LevelFilter::WARN),
                "info" => Some(LevelFilter::INFO),
                "debug" => Some(LevelFilter::DEBUG),
                "trace" => Some(LevelFilter::TRACE),
                "off" => Some(LevelFilter::OFF),
                _ => None,
            },
        )
    }

    fn finish<Loaded>(self, loaded: Option<Loaded>) -> Result<Loaded, InvalidEnvironment> {
        loaded.ok_or_else(|| {
            let mut rejected = self.rejected;
            rejected.sort();
            InvalidEnvironment { rejected }
        })
    }
}

impl Config {
    pub fn from_env() -> Result<Self, InvalidEnvironment> {
        Self::from_source(|name| std::env::var(name).ok())
    }

    pub fn from_source(read: impl Fn(&str) -> Option<String>) -> Result<Self, InvalidEnvironment> {
        let mut reader = Reader {
            read,
            rejected: Vec::new(),
        };

        let allowed_origins = reader.require(
            "ALLOWED_ORIGINS",
            "expected a comma-separated list of absolute URLs with no trailing slash (https://…)",
            |raw| {
                raw.split(',')
                    .map(|origin| absolute_origin(origin.trim().to_string()))
                    .collect()
            },
        );
        let api_url = reader.require("API_URL", ABSOLUTE_URL, absolute_origin);
        let app_env = reader.app_env();
        let app_origin = reader.require("APP_ORIGIN", ABSOLUTE_URL, absolute_origin);
        let client_ip_source = reader.require(
            "CLIENT_IP_SOURCE",
            "expected one of: CfConnectingIp, CloudFrontViewerAddress, ConnectInfo, FlyClientIp, RightmostXForwardedFor, TrueClientIp, XEnvoyExternalAddress, XRealIp",
            |raw| raw.parse::<ClientIpSource>().ok(),
        );
        let database_url = reader.database_url();
        let google_client_id = reader.require("GOOGLE_CLIENT_ID", NON_EMPTY, non_empty);
        let google_client_secret = reader.require("GOOGLE_CLIENT_SECRET", NON_EMPTY, non_empty);
        let jwt_secret = reader.require(
            "JWT_SECRET",
            "expected a string with at least 32 characters",
            |raw| Some(raw).filter(|secret| secret.chars().count() >= 32),
        );
        let log_level = reader.log_level();
        let port = reader.require("PORT", PORT_RANGE, parse_port);
        let redis_url = reader.require(
            "REDIS_URL",
            "expected a Redis connection string (redis://…)",
            |raw| url_with_scheme(raw, &["redis"]).and_then(|raw| raw.into_connection_info().ok()),
        );

        let config = (|| {
            Some(Config {
                allowed_origins: allowed_origins?,
                api_url: api_url?,
                app_env: app_env?,
                app_origin: app_origin?,
                client_ip_source: client_ip_source?,
                database_url: database_url?,
                google_client_id: google_client_id?,
                google_client_secret: google_client_secret?,
                jwt_secret: jwt_secret?,
                log_level: log_level?,
                port: port?,
                redis_url: redis_url?,
            })
        })();
        reader.finish(config)
    }
}

impl WorkerConfig {
    pub fn from_env() -> Result<Self, InvalidEnvironment> {
        Self::from_source(|name| std::env::var(name).ok())
    }

    pub fn from_source(read: impl Fn(&str) -> Option<String>) -> Result<Self, InvalidEnvironment> {
        let mut reader = Reader {
            read,
            rejected: Vec::new(),
        };

        let app_env = reader.app_env();
        let database_url = reader.database_url();
        let log_level = reader.log_level();
        let smtp_user = reader.require("SMTP_USER", "expected an email address", |raw| {
            Some(raw).filter(|user| user.parse::<Address>().is_ok())
        });
        let mail_from = reader.require(
            "MAIL_FROM",
            "expected an email address equal to SMTP_USER",
            |raw| {
                Some(raw)
                    .filter(|from| Some(from) == smtp_user.as_ref())
                    .and_then(|from| from.parse::<Address>().ok())
            },
        );
        let smtp_host = reader.require("SMTP_HOST", "expected a hostname", |raw| {
            Some(raw).filter(|host| is_fqdn(host))
        });
        let smtp_password = reader.require("SMTP_PASSWORD", NON_EMPTY, non_empty);
        let smtp_port = reader.require("SMTP_PORT", PORT_RANGE, parse_port);

        let config = (|| {
            Some(WorkerConfig {
                app_env: app_env?,
                database_url: database_url?,
                log_level: log_level?,
                smtp_user: smtp_user?,
                mail_from: mail_from?,
                smtp_host: smtp_host?,
                smtp_password: smtp_password?,
                smtp_port: smtp_port?,
            })
        })();
        reader.finish(config)
    }
}

fn absolute_origin(raw: String) -> Option<String> {
    let parsed = Url::parse(&raw).ok()?;
    if !["http", "https"].contains(&parsed.scheme()) {
        return None;
    }
    Some(raw).filter(|origin| parsed.origin().ascii_serialization() == *origin)
}

fn url_with_scheme(raw: String, schemes: &[&str]) -> Option<String> {
    let parsed = Url::parse(&raw).ok()?;
    let has_host = parsed.host_str().is_some_and(|host| !host.is_empty());
    (has_host && schemes.contains(&parsed.scheme())).then_some(raw)
}

fn non_empty(raw: String) -> Option<String> {
    Some(raw).filter(|value| !value.is_empty())
}

fn parse_port(raw: String) -> Option<u16> {
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    raw.parse::<u16>().ok().filter(|port| *port >= 1)
}

fn is_fqdn(host: &str) -> bool {
    let labels: Vec<&str> = host.split('.').collect();
    let tld = labels.last().copied().unwrap_or_default();
    let valid_label = |label: &&str| {
        (1..=63).contains(&label.len())
            && label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            && !label.starts_with('-')
            && !label.ends_with('-')
    };
    labels.len() >= 2
        && host.len() <= 253
        && labels.iter().all(valid_label)
        && tld.len() >= 2
        && tld.bytes().all(|byte| byte.is_ascii_alphabetic())
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use super::*;

    const API_ENVIRONMENT: [(&str, &str); 12] = [
        (
            "ALLOWED_ORIGINS",
            "http://localhost:3000, https://app.example.com",
        ),
        ("API_URL", "http://localhost:3333"),
        ("APP_ENV", "development"),
        ("APP_ORIGIN", "http://localhost:3000"),
        ("CLIENT_IP_SOURCE", "RightmostXForwardedFor"),
        ("DATABASE_URL", "postgresql://app:local@localhost:5432/app"),
        ("GOOGLE_CLIENT_ID", "client-id"),
        ("GOOGLE_CLIENT_SECRET", "client-secret"),
        ("JWT_SECRET", "a-jwt-secret-with-at-least-32-characters"),
        ("LOG_LEVEL", "off"),
        ("PORT", "3333"),
        ("REDIS_URL", "redis://localhost:6379"),
    ];

    const WORKER_ENVIRONMENT: [(&str, &str); 8] = [
        ("APP_ENV", "production"),
        ("DATABASE_URL", "postgresql://app:local@localhost:5432/app"),
        ("LOG_LEVEL", "warn"),
        ("MAIL_FROM", "clinicore@example.com"),
        ("SMTP_HOST", "smtp-relay.example.com"),
        ("SMTP_PASSWORD", "an-smtp-password"),
        ("SMTP_PORT", "587"),
        ("SMTP_USER", "clinicore@example.com"),
    ];

    fn source(
        environment: &[(&str, &str)],
        overrides: &[(&str, Option<&str>)],
    ) -> impl Fn(&str) -> Option<String> {
        let mut variables: Vec<(String, String)> = environment
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect();
        for (name, value) in overrides {
            variables.retain(|(existing, _)| existing != name);
            if let Some(value) = value {
                variables.push((name.to_string(), value.to_string()));
            }
        }
        move |name| {
            variables
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.clone())
        }
    }

    fn api_rejection(overrides: &[(&str, Option<&str>)]) -> Vec<&'static str> {
        match Config::from_source(source(&API_ENVIRONMENT, overrides)) {
            Ok(_) => Vec::new(),
            Err(rejection) => rejection.rejected.iter().map(|(name, _)| *name).collect(),
        }
    }

    fn worker_rejection(overrides: &[(&str, Option<&str>)]) -> Vec<&'static str> {
        match WorkerConfig::from_source(source(&WORKER_ENVIRONMENT, overrides)) {
            Ok(_) => Vec::new(),
            Err(rejection) => rejection.rejected.iter().map(|(name, _)| *name).collect(),
        }
    }

    #[test]
    fn a_complete_environment_becomes_typed_values() {
        let config =
            Config::from_source(source(&API_ENVIRONMENT, &[])).expect("a complete environment");

        assert_eq!(
            config.allowed_origins,
            ["http://localhost:3000", "https://app.example.com"]
        );
        assert_eq!(config.app_env, AppEnv::Development);
        assert_eq!(config.log_level, LevelFilter::OFF);
        assert_eq!(config.port, 3333);
        assert_eq!(
            config.client_ip_source,
            ClientIpSource::RightmostXForwardedFor
        );
        assert_eq!(config.database_url.get_database(), Some("app"));

        let worker = WorkerConfig::from_source(source(&WORKER_ENVIRONMENT, &[]))
            .expect("a complete worker environment");
        assert_eq!(worker.app_env, AppEnv::Production);
        assert_eq!(worker.log_level, LevelFilter::WARN);
        assert_eq!(worker.mail_from.to_string(), "clinicore@example.com");
        assert_eq!(worker.smtp_port, 587);
        assert!(worker_rejection(&[]).is_empty());
    }

    #[test]
    fn each_missing_variable_is_named_on_its_own() {
        for (name, _) in API_ENVIRONMENT {
            assert_eq!(api_rejection(&[(name, None)]), [name]);
        }
        for (name, _) in WORKER_ENVIRONMENT {
            let rejected = worker_rejection(&[(name, None)]);
            let expected: &[&str] = match name {
                "SMTP_USER" => &["MAIL_FROM", "SMTP_USER"],
                _ => &[name],
            };
            assert_eq!(rejected, expected, "{name}");
        }
    }

    #[test]
    fn a_malformed_value_is_named_on_its_own() {
        for (name, value) in [
            ("APP_ENV", "staging"),
            ("LOG_LEVEL", "verbose"),
            ("API_URL", "not a url"),
            ("API_URL", "ftp://example.com"),
            ("APP_ORIGIN", "http://localhost:3000/"),
            ("DATABASE_URL", "not a url"),
            ("DATABASE_URL", "mysql://app:local@localhost/app"),
            (
                "DATABASE_URL",
                "postgresql://app@localhost/app?sslmode=sometimes",
            ),
            ("REDIS_URL", "redis://localhost:6379/not-a-number"),
        ] {
            assert_eq!(api_rejection(&[(name, Some(value))]), [name], "{value}");
        }
        for (name, value) in [
            ("SMTP_HOST", "localhost"),
            ("SMTP_HOST", "-relay.example.com"),
            ("SMTP_HOST", "relay-.example.com"),
            ("SMTP_HOST", "relay.example.c0m"),
            ("SMTP_HOST", "relay..example.com"),
            ("MAIL_FROM", "another@example.com"),
        ] {
            assert_eq!(worker_rejection(&[(name, Some(value))]), [name], "{value}");
        }
    }

    #[test]
    fn every_log_level_and_app_env_is_accepted() {
        for level in ["error", "warn", "info", "debug", "trace", "off"] {
            assert!(api_rejection(&[("LOG_LEVEL", Some(level))]).is_empty());
        }
        for app_env in ["development", "production", "test"] {
            assert!(api_rejection(&[("APP_ENV", Some(app_env))]).is_empty());
        }
    }

    struct FailingAfter(usize);

    impl Write for FailingAfter {
        fn write_str(&mut self, _: &str) -> fmt::Result {
            if self.0 == 0 {
                return Err(fmt::Error);
            }
            self.0 -= 1;
            Ok(())
        }
    }

    #[test]
    fn a_failing_writer_fails_the_rejection_message() {
        let rejection = InvalidEnvironment {
            rejected: vec![("PORT", PORT_RANGE)],
        };

        assert_eq!(
            rejection.to_string(),
            format!("Invalid environment:\n  PORT: {PORT_RANGE}")
        );
        for writes_before_failing in [0, 1] {
            assert!(write!(FailingAfter(writes_before_failing), "{rejection}").is_err());
        }
    }
}
