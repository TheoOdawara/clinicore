mod common;

use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use common::environment_with;

struct Exit {
    code: Option<i32>,
    stderr: String,
}

fn boot(overrides: &[(&str, Option<&str>)]) -> Exit {
    let mut child = Command::new(env!("CARGO_BIN_EXE_api"))
        .env_clear()
        .envs(environment_with(overrides))
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the api binary");
    let deadline = Instant::now() + Duration::from_secs(10);
    while child.try_wait().expect("a waitable child").is_none() {
        if Instant::now() > deadline {
            child.kill().expect("a killable child");
            panic!("the api booted instead of refusing the environment");
        }
        thread::sleep(Duration::from_millis(20));
    }
    let output = child.wait_with_output().expect("the child output");
    Exit {
        code: output.status.code(),
        stderr: String::from_utf8(output.stderr).expect("a UTF-8 stderr"),
    }
}

#[test]
fn a_missing_database_url_stops_the_boot() {
    let exit = boot(&[("DATABASE_URL", None)]);

    assert_eq!(exit.code, Some(1));
    assert_eq!(
        exit.stderr,
        "Invalid environment:\n  DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)\n"
    );
}

#[test]
fn every_rejected_variable_is_listed_in_alphabetical_order() {
    let exit = boot(&[("PORT", Some("abc")), ("JWT_SECRET", None)]);

    assert_eq!(exit.code, Some(1));
    assert_eq!(
        exit.stderr,
        "Invalid environment:\n  JWT_SECRET: expected a string with at least 32 characters\n  PORT: expected an integer between 1 and 65535\n"
    );
}

#[test]
fn an_incomplete_environment_names_what_is_missing_and_a_mail_from_that_is_not_the_smtp_user() {
    let exit = boot(&[
        ("JWT_SECRET", None),
        ("TRUSTED_PROXIES", None),
        ("MAIL_FROM", Some("someone-else@example.com")),
    ]);

    assert_eq!(exit.code, Some(1));
    assert_eq!(
        exit.stderr,
        "Invalid environment:\n  JWT_SECRET: expected a string with at least 32 characters\n  MAIL_FROM: expected an email address equal to SMTP_USER\n  TRUSTED_PROXIES: expected a comma-separated list of CIDR blocks (10.0.0.0/8,…)\n"
    );
}
