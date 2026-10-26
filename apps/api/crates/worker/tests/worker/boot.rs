use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn an_incomplete_environment_stops_the_boot_and_names_every_rejected_variable() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_worker"))
        .env_clear()
        .envs([
            ("APP_ENV", "test"),
            ("LOG_LEVEL", "off"),
            ("MAIL_FROM", "test@example.com"),
            ("SMTP_HOST", "smtp.gmail.com"),
            ("SMTP_PORT", "587"),
            ("SMTP_USER", "test@example.com"),
        ])
        .envs(std::env::var_os("LLVM_PROFILE_FILE").map(|file| ("LLVM_PROFILE_FILE", file)))
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the worker binary");
    let deadline = Instant::now() + Duration::from_secs(10);
    while child.try_wait().expect("a waitable child").is_none() {
        if Instant::now() > deadline {
            child.kill().expect("a killable child");
            panic!("the worker booted instead of refusing the environment");
        }
        thread::sleep(Duration::from_millis(20));
    }
    let output = child.wait_with_output().expect("the child output");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr).expect("a UTF-8 stderr"),
        "Invalid environment:\n  DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)\n  SMTP_PASSWORD: expected a non-empty string\n"
    );
}
