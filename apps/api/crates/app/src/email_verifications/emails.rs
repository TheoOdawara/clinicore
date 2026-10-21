use clinicore_core::mail::{MailContent, MailError, MailMessage, compose};

pub fn verification(name: &str, link: &str) -> Result<MailMessage, MailError> {
    let greeting = first_name(name);
    compose(
        "Confirme seu e-mail no Clinicore",
        &MailContent {
            name: &greeting,
            paragraphs: &[
                "Confirme seu e-mail para começar a usar o Clinicore.",
                "O link expira em 1 hora.",
            ],
            action: "Confirmar e-mail",
            link,
        },
    )
}

pub fn first_name(name: &str) -> String {
    name.split_whitespace()
        .next()
        .unwrap_or_default()
        .chars()
        .take(30)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::first_name;

    #[test]
    fn the_greeting_uses_only_the_first_name_capped_at_30_characters() {
        assert_eq!(first_name("Ana Maria Souza"), "Ana");
        assert_eq!(first_name(&"A".repeat(40)), "A".repeat(30));
    }
}
