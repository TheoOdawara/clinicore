use std::fmt;

use axum_client_ip::ClientIpSource;
use email_address::EmailAddress;
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    Off,
}

#[derive(Clone)]
pub struct Config {
    pub allowed_origins: Vec<String>,
    pub api_url: String,
    pub app_env: AppEnv,
    pub app_origin: String,
    pub client_ip_source: ClientIpSource,
    pub database_url: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub jwt_secret: String,
    pub log_level: LogLevel,
    pub mail_from: String,
    pub port: u16,
    pub redis_url: String,
    pub smtp_host: String,
    pub smtp_password: String,
    pub smtp_port: u16,
    pub smtp_user: String,
}

pub struct InvalidEnvironment {
    rejected: Vec<(&'static str, &'static str)>,
}

impl fmt::Display for InvalidEnvironment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut rejected = self.rejected.clone();
        rejected.sort();
        write!(formatter, "Invalid environment:")?;
        for (name, expected) in rejected {
            write!(formatter, "\n  {name}: {expected}")?;
        }
        Ok(())
    }
}

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
            |raw| list_of(&raw, absolute_origin),
        );
        let api_url = reader.require("API_URL", ABSOLUTE_URL, absolute_origin);
        let app_env = reader.require(
            "APP_ENV",
            "expected one of development, production, test",
            |raw| match raw.as_str() {
                "development" => Some(AppEnv::Development),
                "production" => Some(AppEnv::Production),
                "test" => Some(AppEnv::Test),
                _ => None,
            },
        );
        let app_origin = reader.require("APP_ORIGIN", ABSOLUTE_URL, absolute_origin);
        let client_ip_source = reader.require(
            "CLIENT_IP_SOURCE",
            "expected one of: CfConnectingIp, CloudFrontViewerAddress, ConnectInfo, FlyClientIp, RightmostXForwardedFor, TrueClientIp, XEnvoyExternalAddress, XRealIp",
            |raw| raw.parse::<ClientIpSource>().ok(),
        );
        let database_url = reader.require(
            "DATABASE_URL",
            "expected a PostgreSQL connection string (postgresql://…)",
            |raw| url_with_scheme(raw, &["postgresql", "postgres"]),
        );
        let google_client_id = reader.require("GOOGLE_CLIENT_ID", NON_EMPTY, non_empty);
        let google_client_secret = reader.require("GOOGLE_CLIENT_SECRET", NON_EMPTY, non_empty);
        let jwt_secret = reader.require(
            "JWT_SECRET",
            "expected a string with at least 32 characters",
            |raw| Some(raw).filter(|secret| secret.chars().count() >= 32),
        );
        let log_level = reader.require(
            "LOG_LEVEL",
            "expected one of: error, warn, info, debug, trace, off",
            |raw| match raw.as_str() {
                "error" => Some(LogLevel::Error),
                "warn" => Some(LogLevel::Warn),
                "info" => Some(LogLevel::Info),
                "debug" => Some(LogLevel::Debug),
                "trace" => Some(LogLevel::Trace),
                "off" => Some(LogLevel::Off),
                _ => None,
            },
        );
        let smtp_user = reader.require("SMTP_USER", "expected an email address", |raw| {
            Some(raw).filter(|user| EmailAddress::is_valid(user))
        });
        let mail_from = reader.require(
            "MAIL_FROM",
            "expected an email address equal to SMTP_USER",
            |raw| Some(raw).filter(|from| Some(from) == smtp_user.as_ref()),
        );
        let port = reader.require("PORT", PORT_RANGE, parse_port);
        let redis_url = reader.require(
            "REDIS_URL",
            "expected a Redis connection string (redis://…)",
            |raw| url_with_scheme(raw, &["redis"]),
        );
        let smtp_host = reader.require("SMTP_HOST", "expected a hostname", |raw| {
            Some(raw).filter(|host| is_fqdn(host))
        });
        let smtp_password = reader.require("SMTP_PASSWORD", NON_EMPTY, non_empty);
        let smtp_port = reader.require("SMTP_PORT", PORT_RANGE, parse_port);

        let (
            Some(allowed_origins),
            Some(api_url),
            Some(app_env),
            Some(app_origin),
            Some(client_ip_source),
            Some(database_url),
            Some(google_client_id),
            Some(google_client_secret),
            Some(jwt_secret),
            Some(log_level),
            Some(mail_from),
            Some(port),
            Some(redis_url),
            Some(smtp_host),
            Some(smtp_password),
            Some(smtp_port),
            Some(smtp_user),
        ) = (
            allowed_origins,
            api_url,
            app_env,
            app_origin,
            client_ip_source,
            database_url,
            google_client_id,
            google_client_secret,
            jwt_secret,
            log_level,
            mail_from,
            port,
            redis_url,
            smtp_host,
            smtp_password,
            smtp_port,
            smtp_user,
        )
        else {
            return Err(InvalidEnvironment {
                rejected: reader.rejected,
            });
        };

        Ok(Config {
            allowed_origins,
            api_url,
            app_env,
            app_origin,
            client_ip_source,
            database_url,
            google_client_id,
            google_client_secret,
            jwt_secret,
            log_level,
            mail_from,
            port,
            redis_url,
            smtp_host,
            smtp_password,
            smtp_port,
            smtp_user,
        })
    }
}

fn list_of<Item>(raw: &str, parse: impl Fn(String) -> Option<Item>) -> Option<Vec<Item>> {
    raw.split(',')
        .map(|item| parse(item.trim().to_string()))
        .collect()
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
    let Some(tld) = labels.last() else {
        return false;
    };
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
    use super::*;

    #[test]
    fn a_complete_environment_becomes_typed_values() {
        let environment = [
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
            ("MAIL_FROM", "person@example.com"),
            ("PORT", "3333"),
            ("REDIS_URL", "redis://localhost:6379"),
            ("SMTP_HOST", "smtp.gmail.com"),
            ("SMTP_PASSWORD", "password"),
            ("SMTP_PORT", "587"),
            ("SMTP_USER", "person@example.com"),
        ];
        let read = |name: &str| {
            environment
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| value.to_string())
        };

        let Ok(config) = Config::from_source(read) else {
            panic!("a complete environment was refused");
        };

        assert_eq!(
            config.allowed_origins,
            ["http://localhost:3000", "https://app.example.com"]
        );
        assert_eq!(config.app_env, AppEnv::Development);
        assert_eq!(config.log_level, LogLevel::Off);
        assert_eq!(config.port, 3333);
        assert_eq!(config.smtp_port, 587);
        assert_eq!(
            config.client_ip_source,
            ClientIpSource::RightmostXForwardedFor
        );
    }
}
