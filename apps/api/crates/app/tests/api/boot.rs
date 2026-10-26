use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::support::environment_with;

struct Exit {
    code: Option<i32>,
    stderr: String,
}

fn boot(overrides: &[(&str, Option<&str>)]) -> Exit {
    let mut child = Command::new(env!("CARGO_BIN_EXE_api"))
        .env_clear()
        .envs(environment_with(overrides))
        .envs(std::env::var_os("LLVM_PROFILE_FILE").map(|file| ("LLVM_PROFILE_FILE", file)))
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
fn an_invalid_environment_stops_the_boot_and_names_every_rejected_variable() {
    let exit = boot(&[
        ("DATABASE_URL", None),
        ("PORT", Some("abc")),
        ("JWT_SECRET", Some("too-short")),
    ]);

    assert_eq!(exit.code, Some(1));
    assert_eq!(
        exit.stderr,
        "Invalid environment:\n  DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)\n  JWT_SECRET: expected a string with at least 32 characters\n  PORT: expected an integer between 1 and 65535\n"
    );
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("[::]:0")
        .expect("a free port")
        .local_addr()
        .expect("a bound address")
        .port()
}

fn answers_http(port: u16) -> bool {
    let Ok(mut stream) = std::net::TcpStream::connect(("::1", port)) else {
        return false;
    };
    let sent =
        stream.write_all(b"GET /nowhere HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    let mut response = String::new();
    sent.is_ok()
        && stream.read_to_string(&mut response).is_ok()
        && response.starts_with("HTTP/1.1 404")
}

#[test]
fn a_signal_stops_the_api_gracefully() {
    for signal in ["TERM", "INT"] {
        let port = free_port().to_string();
        let mut child = Command::new(env!("CARGO_BIN_EXE_api"))
            .env_clear()
            .envs(environment_with(&[
                ("APP_ENV", Some("development")),
                ("PORT", Some(&port)),
            ]))
            .envs(std::env::var_os("LLVM_PROFILE_FILE").map(|file| ("LLVM_PROFILE_FILE", file)))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("the api binary");
        let port = port.parse().expect("a port");
        let deadline = Instant::now() + Duration::from_secs(10);
        while !answers_http(port) {
            if Instant::now() > deadline {
                child.kill().expect("a killable child");
                panic!("the api never answered");
            }
            thread::sleep(Duration::from_millis(20));
        }

        let killed = Command::new("kill")
            .args(["-s", signal, &child.id().to_string()])
            .status()
            .expect("the kill command");
        assert!(killed.success());
        let deadline = Instant::now() + Duration::from_secs(10);
        while child.try_wait().expect("a waitable child").is_none() {
            if Instant::now() > deadline {
                child.kill().expect("a killable child");
                panic!("SIG{signal} did not stop the api");
            }
            thread::sleep(Duration::from_millis(20));
        }

        assert_eq!(
            child.wait().expect("the exit").code(),
            Some(0),
            "SIG{signal}"
        );
    }
}

#[test]
fn a_port_already_in_use_stops_the_boot() {
    let taken = std::net::TcpListener::bind("[::]:0").expect("a free port");
    let port = taken
        .local_addr()
        .expect("a bound address")
        .port()
        .to_string();

    let exit = boot(&[("PORT", Some(&port))]);

    assert_eq!(exit.code, Some(1));
}
