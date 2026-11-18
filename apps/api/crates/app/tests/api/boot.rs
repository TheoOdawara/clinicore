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
            "postgresql://clinicore:secret-value@localhost:5432/clinicore",
        )
        .env("REDIS_URL", "redis://localhost:6379")
        .env("ALLOWED_ORIGINS", "http://localhost:3000")
        .env("APP_ORIGIN", "https://secret-value.example.com")
        .env("CLIENT_IP_SOURCE", "peer");
    if let Ok(profile) = std::env::var("LLVM_PROFILE_FILE") {
        command.env("LLVM_PROFILE_FILE", profile);
    }

    let output = command.output().expect("the api binary runs");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "Invalid environment:\n  \
         APP_ORIGIN: expected an origin listed in ALLOWED_ORIGINS\n  \
         JWT_SECRET: expected a string with at least 32 characters\n"
    );
    assert!(output.stdout.is_empty());
}
