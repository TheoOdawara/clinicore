use clinicore_core::lifetimes::{VERIFICATION_LIFETIME, hours_text};
use clinicore_core::mail::{MailContent, MailMessage, compose};

use crate::email_verifications::emails::first_name;

pub fn password_reset(name: &str, link: &str) -> MailMessage {
    let greeting = first_name(name);
    compose(
        "Redefinir sua senha do Clinicore",
        &MailContent {
            name: &greeting,
            paragraphs: &[
                "Recebemos um pedido para redefinir a sua senha do Clinicore.",
                &format!(
                    "O link expira em {}. Se não foi você, ignore este e-mail.",
                    hours_text(VERIFICATION_LIFETIME)
                ),
            ],
            action: "Redefinir senha",
            link,
        },
    )
}
