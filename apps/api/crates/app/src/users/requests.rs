use unicode_normalization::UnicodeNormalization;
use utoipa::ToSchema;

use crate::credentials::email::Email;
use crate::credentials::password;
use crate::http::validation::{Fields, Validate};

const NAME_MAX_LENGTH: usize = 100;
const NAME_PUNCTUATION: [char; 4] = [' ', '.', '-', '\''];

#[derive(ToSchema)]
#[schema(example = json!({ "name": "Ana Souza", "email": "ana@exemplo.com", "password": "Clinica#2026" }))]
pub struct SignUpRequest {
    pub name: String,
    #[schema(value_type = String)]
    pub email: Email,
    pub password: String,
}

impl Validate for SignUpRequest {
    fn validate(fields: &mut Fields) -> Option<Self> {
        let name = fields.field("name", parse_name);
        let email = fields.field("email", parse_email);
        let password = fields.field("password", parse_password);
        Some(Self {
            name: name?,
            email: email?,
            password: password?,
        })
    }
}

pub fn parse_email(raw: String) -> Result<Email, &'static str> {
    Email::parse(&raw).ok_or("email")
}

fn parse_name(raw: String) -> Result<String, &'static str> {
    let name: String = raw.trim().nfc().collect();
    if !(1..=NAME_MAX_LENGTH).contains(&name.chars().count()) {
        return Err("length");
    }
    let allowed = name
        .chars()
        .all(|character| character.is_alphabetic() || NAME_PUNCTUATION.contains(&character));
    if !allowed || !name.chars().any(char::is_alphabetic) {
        return Err("name");
    }
    Ok(name)
}

fn parse_password(raw: String) -> Result<String, &'static str> {
    if password::meets_policy(&raw) {
        Ok(raw)
    } else {
        Err("weak_password")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_trimmed_and_normalized_to_nfc() {
        let decomposed = "  Joa\u{0303}o D'Ávila-Sá Jr. ";

        assert_eq!(
            parse_name(decomposed.to_owned()),
            Ok("João D'Ávila-Sá Jr.".to_owned())
        );
    }

    #[test]
    fn a_name_has_from_1_to_100_characters() {
        assert_eq!(parse_name("   ".to_owned()), Err("length"));
        assert!(parse_name("Á".repeat(100)).is_ok());
        assert_eq!(parse_name("Á".repeat(101)), Err("length"));
    }

    #[test]
    fn a_name_has_only_letters_and_the_allowed_punctuation_with_one_letter() {
        assert_eq!(parse_name("Ana 2".to_owned()), Err("name"));
        assert_eq!(parse_name("Ana@".to_owned()), Err("name"));
        assert_eq!(parse_name(".-'".to_owned()), Err("name"));
    }
}
