use askama::Template;

pub const EMAIL_VERIFICATION_SUBJECT: &str = "Confirme seu e-mail no Clinicore";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedEmail {
    pub subject: String,
    pub html: String,
    pub text: String,
}

#[derive(Template)]
#[template(path = "email_verification.html")]
struct EmailVerificationHtml<'a> {
    subject: &'a str,
    first_name: &'a str,
    link: &'a str,
}

#[derive(Template)]
#[template(path = "email_verification.txt")]
struct EmailVerificationText<'a> {
    first_name: &'a str,
    link: &'a str,
}

pub fn email_verification(first_name: &str, link: &str) -> RenderedEmail {
    let html = EmailVerificationHtml {
        subject: EMAIL_VERIFICATION_SUBJECT,
        first_name,
        link,
    }
    .render()
    .expect("the email verification template renders from plain strings");
    let text = EmailVerificationText { first_name, link }
        .render()
        .expect("the email verification template renders from plain strings");
    RenderedEmail {
        subject: EMAIL_VERIFICATION_SUBJECT.to_owned(),
        html,
        text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINK: &str = "http://localhost:3000/verify-email?token=abc";

    #[test]
    fn the_subject_is_the_fixed_one() {
        let email = email_verification("Ana", LINK);

        assert_eq!(email.subject, "Confirme seu e-mail no Clinicore");
    }

    #[test]
    fn the_text_greets_by_first_name_and_puts_the_link_on_its_own_line() {
        let email = email_verification("Ana", LINK);

        assert!(email.text.starts_with(
            "Olá, Ana. Confirme seu e-mail para começar a usar o Clinicore. O link expira em 1 hora."
        ));
        assert!(
            email.text.lines().any(|line| line == LINK),
            "{}",
            email.text
        );
    }

    #[test]
    fn the_html_has_the_button_pointing_at_the_link() {
        let email = email_verification("Ana", LINK);

        assert!(email.html.contains("Olá, Ana."));
        assert!(email.html.contains(&format!(r#"href="{LINK}""#)));
        assert!(email.html.contains(">Confirmar e-mail</a>"));
    }

    #[test]
    fn the_html_escapes_the_name() {
        let email = email_verification("<b>Ana</b>", LINK);

        assert!(!email.html.contains("<b>Ana</b>"));
        assert!(email.html.contains("Olá, &#60;b&#62;Ana&#60;/b&#62;."));
    }
}
