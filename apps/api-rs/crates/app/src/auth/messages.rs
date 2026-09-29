use clinicore_core::mail::{MailContent, MailMessage, compose};

use crate::http::error::AppError;

pub fn email_verification(name: &str, link: &str) -> Result<MailMessage, AppError> {
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
    .map_err(AppError::internal)
}
