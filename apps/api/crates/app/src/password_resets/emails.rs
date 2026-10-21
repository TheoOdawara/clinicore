use clinicore_core::mail::{MailContent, MailError, MailMessage, compose};

use crate::email_verifications::emails::first_name;

pub fn password_reset(name: &str, link: &str) -> Result<MailMessage, MailError> {
    let greeting = first_name(name);
    compose(
        "Redefinir sua senha do Clinicore",
        &MailContent {
            name: &greeting,
            paragraphs: &[
                "Recebemos um pedido para redefinir a sua senha do Clinicore.",
                "O link expira em 1 hora. Se não foi você, ignore este e-mail.",
            ],
            action: "Redefinir senha",
            link,
        },
    )
}
