use clinicore_worker::mail::Mailer;

use crate::support::config;

#[tokio::test]
async fn every_smtp_tls_mode_builds_a_mailer_without_connecting() {
    for tls in ["starttls", "tls", "none"] {
        let built = Mailer::new(&config(1, &[("SMTP_TLS", tls)]));

        assert!(built.is_ok(), "{tls}");
    }
}

#[tokio::test]
async fn a_sender_lettre_cannot_parse_is_refused_at_boot() {
    let built = Mailer::new(&config(
        1,
        &[("MAIL_FROM", "Clinicore <a..b@clinicore.com.br>")],
    ));

    assert!(built.is_err());
}
