use std::fmt;

use tracing::level_filters::LevelFilter;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppEnv {
    Development,
    Production,
    Test,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientIpSource {
    Peer,
    CfConnectingIp,
    XForwardedFor(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmtpTls {
    Starttls,
    Tls,
    None,
}

pub struct ApiConfig {
    pub app_env: AppEnv,
    pub log_level: LevelFilter,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub allowed_origins: Vec<String>,
    pub app_origin: String,
    pub client_ip_source: ClientIpSource,
    pub jwt_secret: String,
}

pub struct WorkerConfig {
    pub app_env: AppEnv,
    pub log_level: LevelFilter,
    pub database_url: String,
    pub mail_from: String,
    pub mail_redirect_to: Option<String>,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_tls: SmtpTls,
    pub smtp_user: String,
    pub smtp_password: String,
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

const ALLOWED_ORIGINS: &str = "ALLOWED_ORIGINS";
const APP_ENV: &str = "APP_ENV";
const APP_ORIGIN: &str = "APP_ORIGIN";
const CLIENT_IP_SOURCE: &str = "CLIENT_IP_SOURCE";
const DATABASE_URL: &str = "DATABASE_URL";
const JWT_SECRET: &str = "JWT_SECRET";
const LOG_LEVEL: &str = "LOG_LEVEL";
const MAIL_FROM: &str = "MAIL_FROM";
const MAIL_REDIRECT_TO: &str = "MAIL_REDIRECT_TO";
const PORT: &str = "PORT";
const REDIS_URL: &str = "REDIS_URL";
const SMTP_HOST: &str = "SMTP_HOST";
const SMTP_PASSWORD: &str = "SMTP_PASSWORD";
const SMTP_PORT: &str = "SMTP_PORT";
const SMTP_TLS: &str = "SMTP_TLS";
const SMTP_USER: &str = "SMTP_USER";

const JWT_SECRET_MIN_LENGTH: usize = 32;
const MAX_TRUSTED_PROXIES: usize = 5;

struct Reader<F> {
    read: F,
    rejected: Vec<(&'static str, &'static str)>,
}

impl<F: Fn(&str) -> Option<String>> Reader<F> {
    fn new(read: F) -> Self {
        Self {
            read,
            rejected: Vec::new(),
        }
    }

    fn field<T>(
        &mut self,
        name: &'static str,
        expected: &'static str,
        parse: impl FnOnce(String) -> Option<T>,
    ) -> Option<T> {
        let value = (self.read)(name).and_then(parse);
        if value.is_none() {
            self.rejected.push((name, expected));
        }
        value
    }

    fn app_env(&mut self) -> Option<AppEnv> {
        self.field(
            APP_ENV,
            "expected one of development, production, test",
            |value| match value.as_str() {
                "development" => Some(AppEnv::Development),
                "production" => Some(AppEnv::Production),
                "test" => Some(AppEnv::Test),
                _ => None,
            },
        )
    }

    fn log_level(&mut self) -> Option<LevelFilter> {
        self.field(
            LOG_LEVEL,
            "expected one of error, warn, info, debug, trace, off",
            |value| match value.as_str() {
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

    fn port(&mut self, name: &'static str) -> Option<u16> {
        self.field(name, "expected an integer between 1 and 65535", |value| {
            value.parse::<u16>().ok().filter(|port| *port != 0)
        })
    }

    fn database_url(&mut self) -> Option<String> {
        self.field(
            DATABASE_URL,
            "expected a PostgreSQL connection string (postgresql://…)",
            |value| has_scheme(&value, &["postgresql://", "postgres://"]).then_some(value),
        )
    }

    fn non_empty(&mut self, name: &'static str) -> Option<String> {
        self.field(name, "expected a non-empty string", |value| {
            (!value.is_empty()).then_some(value)
        })
    }

    fn finish<T>(mut self, config: Option<T>) -> Result<T, InvalidEnvironment> {
        match config {
            Some(config) if self.rejected.is_empty() => Ok(config),
            _ => {
                self.rejected.sort_by_key(|(name, _)| *name);
                Err(InvalidEnvironment {
                    rejected: self.rejected,
                })
            }
        }
    }
}

impl ApiConfig {
    pub fn from_env() -> Result<Self, InvalidEnvironment> {
        Self::from_source(|name| std::env::var(name).ok())
    }

    pub fn from_source(read: impl Fn(&str) -> Option<String>) -> Result<Self, InvalidEnvironment> {
        let mut reader = Reader::new(read);
        let app_env = reader.app_env();
        let log_level = reader.log_level();
        let port = reader.port(PORT);
        let database_url = reader.database_url();
        let redis_url = reader.field(
            REDIS_URL,
            "expected a Redis connection string (redis://…)",
            |value| has_scheme(&value, &["redis://", "rediss://"]).then_some(value),
        );
        let allowed_origins = reader.field(
            ALLOWED_ORIGINS,
            "expected a comma-separated list of origins (https://host[:port])",
            |value| {
                value
                    .split(',')
                    .map(|origin| is_origin(origin).then(|| origin.to_owned()))
                    .collect::<Option<Vec<_>>>()
            },
        );
        let app_origin = match &allowed_origins {
            Some(origins) => reader.field(
                APP_ORIGIN,
                "expected an origin listed in ALLOWED_ORIGINS",
                |value| origins.contains(&value).then_some(value),
            ),
            None => reader.field(
                APP_ORIGIN,
                "expected an origin listed in ALLOWED_ORIGINS",
                |value| is_origin(&value).then_some(value),
            ),
        };
        let client_ip_source = reader.field(
            CLIENT_IP_SOURCE,
            "expected one of peer, cf-connecting-ip, x-forwarded-for:<1-5>",
            |value| parse_client_ip_source(&value),
        );
        let jwt_secret = reader.field(
            JWT_SECRET,
            "expected a string with at least 32 characters",
            |value| (value.chars().count() >= JWT_SECRET_MIN_LENGTH).then_some(value),
        );

        let config = (|| {
            Some(Self {
                app_env: app_env?,
                log_level: log_level?,
                port: port?,
                database_url: database_url?,
                redis_url: redis_url?,
                allowed_origins: allowed_origins?,
                app_origin: app_origin?,
                client_ip_source: client_ip_source?,
                jwt_secret: jwt_secret?,
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
        let mut reader = Reader::new(read);
        let app_env = reader.app_env();
        let log_level = reader.log_level();
        let database_url = reader.database_url();
        let mail_from = reader.field(
            MAIL_FROM,
            "expected a mailbox (Name <address@domain>)",
            |value| is_mailbox(&value).then_some(value),
        );
        let mail_redirect_to = reader.field(
            MAIL_REDIRECT_TO,
            "expected off, or an address outside production",
            |value| match value.as_str() {
                "off" => Some(None),
                _ if app_env != Some(AppEnv::Production) && is_address(&value) => Some(Some(value)),
                _ => None,
            },
        );
        let smtp_host = reader.field(SMTP_HOST, "expected a hostname", |value| {
            is_hostname(&value).then_some(value)
        });
        let smtp_port = reader.port(SMTP_PORT);
        let smtp_tls = reader.field(
            SMTP_TLS,
            "expected one of starttls, tls, none; none only outside production",
            |value| match value.as_str() {
                "starttls" => Some(SmtpTls::Starttls),
                "tls" => Some(SmtpTls::Tls),
                "none" if app_env != Some(AppEnv::Production) => Some(SmtpTls::None),
                _ => None,
            },
        );
        let smtp_user = reader.non_empty(SMTP_USER);
        let smtp_password = reader.non_empty(SMTP_PASSWORD);

        let config = (|| {
            Some(Self {
                app_env: app_env?,
                log_level: log_level?,
                database_url: database_url?,
                mail_from: mail_from?,
                mail_redirect_to: mail_redirect_to?,
                smtp_host: smtp_host?,
                smtp_port: smtp_port?,
                smtp_tls: smtp_tls?,
                smtp_user: smtp_user?,
                smtp_password: smtp_password?,
            })
        })();
        reader.finish(config)
    }
}

fn has_scheme(value: &str, schemes: &[&str]) -> bool {
    schemes
        .iter()
        .any(|scheme| value.len() > scheme.len() && value.starts_with(scheme))
}

fn is_origin(value: &str) -> bool {
    let Some(authority) = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"))
    else {
        return false;
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (host, Some(port)),
        None => (authority, None),
    };
    is_hostname(host) && port.is_none_or(|port| port.parse::<u16>().is_ok_and(|port| port != 0))
}

fn is_hostname(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && value.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '-')
        })
}

fn is_mailbox(value: &str) -> bool {
    let Some((name, address)) = value
        .strip_suffix('>')
        .and_then(|rest| rest.rsplit_once(" <"))
    else {
        return false;
    };
    !name.trim().is_empty() && is_address(address)
}

fn is_address(value: &str) -> bool {
    let Some((local, domain)) = value.rsplit_once('@') else {
        return false;
    };
    !local.is_empty() && !local.contains(char::is_whitespace) && is_hostname(domain)
}

fn parse_client_ip_source(value: &str) -> Option<ClientIpSource> {
    match value {
        "peer" => Some(ClientIpSource::Peer),
        "cf-connecting-ip" => Some(ClientIpSource::CfConnectingIp),
        _ => value
            .strip_prefix("x-forwarded-for:")
            .and_then(|hops| hops.parse::<usize>().ok())
            .filter(|hops| (1..=MAX_TRUSTED_PROXIES).contains(hops))
            .map(ClientIpSource::XForwardedFor),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    const SECRET: &str = "a-test-secret-with-at-least-32-characters";

    fn api() -> HashMap<&'static str, &'static str> {
        HashMap::from([
            (APP_ENV, "test"),
            (LOG_LEVEL, "warn"),
            (PORT, "3333"),
            (
                DATABASE_URL,
                "postgresql://clinicore:secret-value@localhost:5432/clinicore",
            ),
            (REDIS_URL, "redis://localhost:6379"),
            (
                ALLOWED_ORIGINS,
                "http://localhost:3000,https://app.clinicore.com.br",
            ),
            (APP_ORIGIN, "http://localhost:3000"),
            (CLIENT_IP_SOURCE, "peer"),
            (JWT_SECRET, SECRET),
        ])
    }

    fn worker() -> HashMap<&'static str, &'static str> {
        HashMap::from([
            (APP_ENV, "test"),
            (LOG_LEVEL, "warn"),
            (
                DATABASE_URL,
                "postgresql://clinicore:secret-value@localhost:5432/clinicore",
            ),
            (MAIL_FROM, "Clinicore <nao-responda@clinicore.com.br>"),
            (MAIL_REDIRECT_TO, "off"),
            (SMTP_HOST, "localhost"),
            (SMTP_PORT, "1025"),
            (SMTP_TLS, "none"),
            (SMTP_USER, "clinicore"),
            (SMTP_PASSWORD, "secret-value"),
        ])
    }

    fn load_api(
        source: &HashMap<&'static str, &'static str>,
    ) -> Result<ApiConfig, InvalidEnvironment> {
        ApiConfig::from_source(|name| source.get(name).map(|value| (*value).to_owned()))
    }

    fn load_worker(
        source: &HashMap<&'static str, &'static str>,
    ) -> Result<WorkerConfig, InvalidEnvironment> {
        WorkerConfig::from_source(|name| source.get(name).map(|value| (*value).to_owned()))
    }

    fn rejected_message(error: Option<InvalidEnvironment>) -> String {
        error.expect("the environment is invalid").to_string()
    }

    fn assert_only_rejected(message: &str, name: &str) {
        assert!(
            message.contains(&format!("\n  {name}: expected")),
            "{name} is listed in {message}"
        );
        assert_eq!(
            message.lines().count(),
            2,
            "only {name} is listed in {message}"
        );
        assert!(
            !message.contains("secret-value"),
            "the value of {name} is not echoed"
        );
    }

    #[test]
    fn a_complete_api_environment_is_accepted_without_any_mail_variable() {
        let config = load_api(&api()).expect("the environment is valid");

        assert_eq!(config.app_env, AppEnv::Test);
        assert_eq!(config.log_level, LevelFilter::WARN);
        assert_eq!(config.port, 3333);
        assert_eq!(
            config.database_url,
            "postgresql://clinicore:secret-value@localhost:5432/clinicore"
        );
        assert_eq!(config.redis_url, "redis://localhost:6379");
        assert_eq!(
            config.allowed_origins,
            ["http://localhost:3000", "https://app.clinicore.com.br"]
        );
        assert_eq!(config.app_origin, "http://localhost:3000");
        assert_eq!(config.client_ip_source, ClientIpSource::Peer);
        assert_eq!(config.jwt_secret, SECRET);
    }

    #[test]
    fn each_app_env_and_log_level_is_accepted() {
        for (app_env, expected) in [
            ("development", AppEnv::Development),
            ("production", AppEnv::Production),
            ("test", AppEnv::Test),
        ] {
            let mut source = api();
            source.insert(APP_ENV, app_env);
            assert_eq!(load_api(&source).map(|config| config.app_env), Ok(expected));
        }
        for (log_level, expected) in [
            ("error", LevelFilter::ERROR),
            ("warn", LevelFilter::WARN),
            ("info", LevelFilter::INFO),
            ("debug", LevelFilter::DEBUG),
            ("trace", LevelFilter::TRACE),
            ("off", LevelFilter::OFF),
        ] {
            let mut source = api();
            source.insert(LOG_LEVEL, log_level);
            assert_eq!(
                load_api(&source).map(|config| config.log_level),
                Ok(expected)
            );
        }
    }

    #[test]
    fn each_client_ip_source_is_accepted() {
        for (value, expected) in [
            ("peer", ClientIpSource::Peer),
            ("cf-connecting-ip", ClientIpSource::CfConnectingIp),
            ("x-forwarded-for:1", ClientIpSource::XForwardedFor(1)),
            ("x-forwarded-for:5", ClientIpSource::XForwardedFor(5)),
        ] {
            let mut source = api();
            source.insert(CLIENT_IP_SOURCE, value);
            assert_eq!(
                load_api(&source).map(|config| config.client_ip_source),
                Ok(expected)
            );
        }
    }

    #[test]
    fn the_postgres_and_redis_scheme_aliases_are_accepted() {
        let mut source = api();
        source.insert(DATABASE_URL, "postgres://localhost/clinicore");
        source.insert(REDIS_URL, "rediss://localhost:6380");

        assert!(load_api(&source).is_ok());
    }

    #[test]
    fn an_empty_api_environment_lists_every_variable_in_alphabetical_order() {
        let message = rejected_message(load_api(&HashMap::new()).err());

        assert_eq!(
            message,
            "Invalid environment:\n  \
             ALLOWED_ORIGINS: expected a comma-separated list of origins (https://host[:port])\n  \
             APP_ENV: expected one of development, production, test\n  \
             APP_ORIGIN: expected an origin listed in ALLOWED_ORIGINS\n  \
             CLIENT_IP_SOURCE: expected one of peer, cf-connecting-ip, x-forwarded-for:<1-5>\n  \
             DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)\n  \
             JWT_SECRET: expected a string with at least 32 characters\n  \
             LOG_LEVEL: expected one of error, warn, info, debug, trace, off\n  \
             PORT: expected an integer between 1 and 65535\n  \
             REDIS_URL: expected a Redis connection string (redis://…)"
        );
    }

    #[test]
    fn a_missing_secret_and_an_unlisted_app_origin_are_listed_alone() {
        let mut source = api();
        source.remove(JWT_SECRET);
        source.insert(APP_ORIGIN, "https://evil.example.com");

        assert_eq!(
            rejected_message(load_api(&source).err()),
            "Invalid environment:\n  \
             APP_ORIGIN: expected an origin listed in ALLOWED_ORIGINS\n  \
             JWT_SECRET: expected a string with at least 32 characters"
        );
    }

    #[test]
    fn invalid_api_values_are_refused_without_echoing_them() {
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
            (REDIS_URL, "memcached://secret-value@localhost"),
            (ALLOWED_ORIGINS, "http://localhost:3000,*"),
            (ALLOWED_ORIGINS, "http://localhost:3000/"),
            (ALLOWED_ORIGINS, "ftp://secret-value"),
            (ALLOWED_ORIGINS, "http://localhost:0"),
            (ALLOWED_ORIGINS, ""),
            (CLIENT_IP_SOURCE, "x-forwarded-for:0"),
            (CLIENT_IP_SOURCE, "x-forwarded-for:6"),
            (CLIENT_IP_SOURCE, "x-real-ip"),
            (JWT_SECRET, "short-secret-value"),
        ] {
            let mut source = api();
            source.insert(name, value);
            let message = rejected_message(load_api(&source).err());

            assert_only_rejected(&message, name);
        }
    }

    #[test]
    fn an_invalid_origin_list_does_not_also_reject_the_app_origin() {
        let mut source = api();
        source.insert(ALLOWED_ORIGINS, "not-an-origin");

        let message = rejected_message(load_api(&source).err());

        assert_only_rejected(&message, ALLOWED_ORIGINS);
    }

    #[test]
    fn a_complete_worker_environment_is_accepted() {
        let config = load_worker(&worker()).expect("the environment is valid");

        assert_eq!(config.app_env, AppEnv::Test);
        assert_eq!(config.log_level, LevelFilter::WARN);
        assert_eq!(
            config.mail_from,
            "Clinicore <nao-responda@clinicore.com.br>"
        );
        assert_eq!(config.mail_redirect_to, None);
        assert_eq!(config.smtp_host, "localhost");
        assert_eq!(config.smtp_port, 1025);
        assert_eq!(config.smtp_tls, SmtpTls::None);
        assert_eq!(config.smtp_user, "clinicore");
        assert_eq!(config.smtp_password, "secret-value");
        assert_eq!(
            config.database_url,
            "postgresql://clinicore:secret-value@localhost:5432/clinicore"
        );
    }

    #[test]
    fn each_smtp_tls_mode_is_accepted_in_production_except_none() {
        for (value, expected) in [("starttls", SmtpTls::Starttls), ("tls", SmtpTls::Tls)] {
            let mut source = worker();
            source.insert(APP_ENV, "production");
            source.insert(SMTP_TLS, value);
            assert_eq!(
                load_worker(&source).map(|config| config.smtp_tls),
                Ok(expected)
            );
        }

        let mut source = worker();
        source.insert(APP_ENV, "production");
        let message = rejected_message(load_worker(&source).err());

        assert_eq!(
            message,
            "Invalid environment:\n  \
             SMTP_TLS: expected one of starttls, tls, none; none only outside production"
        );
    }

    #[test]
    fn an_empty_worker_environment_lists_every_variable_in_alphabetical_order() {
        let message = rejected_message(load_worker(&HashMap::new()).err());

        assert_eq!(
            message,
            "Invalid environment:\n  \
             APP_ENV: expected one of development, production, test\n  \
             DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)\n  \
             LOG_LEVEL: expected one of error, warn, info, debug, trace, off\n  \
             MAIL_FROM: expected a mailbox (Name <address@domain>)\n  \
             MAIL_REDIRECT_TO: expected off, or an address outside production\n  \
             SMTP_HOST: expected a hostname\n  \
             SMTP_PASSWORD: expected a non-empty string\n  \
             SMTP_PORT: expected an integer between 1 and 65535\n  \
             SMTP_TLS: expected one of starttls, tls, none; none only outside production\n  \
             SMTP_USER: expected a non-empty string"
        );
    }

    #[test]
    fn invalid_worker_values_are_refused_without_echoing_them() {
        for (name, value) in [
            (MAIL_FROM, "secret-value@clinicore.com.br"),
            (MAIL_FROM, "Clinicore <secret-value>"),
            (MAIL_FROM, " <secret-value@clinicore.com.br>"),
            (MAIL_FROM, "Clinicore <secret value@clinicore.com.br>"),
            (MAIL_FROM, "Clinicore <secret-value@-bad.com>"),
            (MAIL_FROM, "Clinicore <@clinicore.com.br>"),
            (MAIL_REDIRECT_TO, "secret-value"),
            (
                MAIL_REDIRECT_TO,
                "Clinicore <secret-value@clinicore.com.br>",
            ),
            (SMTP_HOST, "secret-value..com"),
            (SMTP_HOST, ""),
            (SMTP_PORT, "0"),
            (SMTP_TLS, "ssl-secret-value"),
            (SMTP_USER, ""),
            (SMTP_PASSWORD, ""),
        ] {
            let mut source = worker();
            source.insert(name, value);
            let message = rejected_message(load_worker(&source).err());

            assert_only_rejected(&message, name);
        }
    }

    #[test]
    fn the_mail_redirect_is_an_address_outside_production_or_off() {
        let mut source = worker();
        source.insert(MAIL_REDIRECT_TO, "dev@exemplo.com");
        assert_eq!(
            load_worker(&source).map(|config| config.mail_redirect_to),
            Ok(Some("dev@exemplo.com".to_owned()))
        );

        source.insert(APP_ENV, "production");
        source.insert(SMTP_TLS, "tls");
        let message = rejected_message(load_worker(&source).err());
        assert_eq!(
            message,
            "Invalid environment:\n  \
             MAIL_REDIRECT_TO: expected off, or an address outside production"
        );

        source.insert(MAIL_REDIRECT_TO, "off");
        assert_eq!(
            load_worker(&source).map(|config| config.mail_redirect_to),
            Ok(None)
        );
    }
}
