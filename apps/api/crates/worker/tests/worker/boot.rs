use std::process::Command;

#[test]
fn an_invalid_environment_stops_the_boot_listing_each_variable() {
    let mut command = Command::new(env!("CARGO_BIN_EXE_worker"));
    command
        .env_clear()
        .env("APP_ENV", "production")
        .env("LOG_LEVEL", "warn")
        .env(
            "DATABASE_URL",
            "postgresql://clinicore:secret-value@localhost:5432/clinicore",
        )
        .env("MAIL_FROM", "Clinicore <nao-responda@clinicore.com.br>")
        .env("MAIL_REDIRECT_TO", "secret-value@exemplo.com")
        .env("SMTP_HOST", "localhost")
        .env("SMTP_PORT", "587")
        .env("SMTP_TLS", "none")
        .env("SMTP_USER", "clinicore");
    if let Ok(profile) = std::env::var("LLVM_PROFILE_FILE") {
        command.env("LLVM_PROFILE_FILE", profile);
    }

    let output = command.output().expect("the worker binary runs");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "Invalid environment:\n  \
         MAIL_REDIRECT_TO: expected off, or an address outside production\n  \
         SMTP_PASSWORD: expected a non-empty string\n  \
         SMTP_TLS: expected one of starttls, tls, none; none only outside production\n"
    );
    assert!(output.stdout.is_empty());
}
