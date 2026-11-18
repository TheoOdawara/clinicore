use std::process::Command;

#[test]
fn an_invalid_environment_stops_the_boot_listing_each_variable() {
    let mut command = Command::new(env!("CARGO_BIN_EXE_api"));
    command
        .env_clear()
        .env("APP_ENV", "test")
        .env("LOG_LEVEL", "warn")
        .env("PORT", "3333")
        .env(
            "DATABASE_URL",
            "mysql://root:secret-value@localhost/clinicore",
        );
    if let Ok(profile) = std::env::var("LLVM_PROFILE_FILE") {
        command.env("LLVM_PROFILE_FILE", profile);
    }

    let output = command.output().expect("the api binary runs");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "Invalid environment:\n  DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)\n"
    );
    assert!(output.stdout.is_empty());
}
