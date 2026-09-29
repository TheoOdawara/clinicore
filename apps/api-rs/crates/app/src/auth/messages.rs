use clinicore_core::mail::{MailContent, MailError, MailMessage, compose};

pub fn email_verification(name: &str, link: &str) -> Result<MailMessage, MailError> {
    compose(
        "Confirme seu e-mail no Clinicore",
        &MailContent {
            name,
            paragraphs: &[
                "Confirme seu e-mail para começar a usar o Clinicore.",
                "O link expira em 1 hora.",
            ],
            action: "Confirmar e-mail",
            link,
        },
    )
}
