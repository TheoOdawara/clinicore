use email_address::{EmailAddress, Options};

const MAX_LENGTH: usize = 254;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn parse(raw: &str) -> Option<Self> {
        let address = raw.trim().to_lowercase();
        if address.chars().count() > MAX_LENGTH {
            return None;
        }
        let options = Options::default()
            .with_required_tld()
            .without_display_text()
            .without_domain_literal();
        EmailAddress::parse_with_options(&address, options).ok()?;
        Some(Self(address))
    }

    pub fn parse_field(raw: String) -> Result<Self, &'static str> {
        Self::parse(&raw).ok_or("email")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_address_is_trimmed_and_lowercased() {
        let email = Email::parse("  Ana@Exemplo.COM ").expect("the address is valid");

        assert_eq!(email.as_str(), "ana@exemplo.com");
    }

    #[test]
    fn a_malformed_address_is_refused() {
        for raw in [
            "",
            "ana",
            "ana@",
            "@exemplo.com",
            "ana@exemplo",
            "Ana <ana@exemplo.com>",
            "ana@[127.0.0.1]",
        ] {
            assert_eq!(Email::parse(raw), None, "{raw}");
        }
    }

    #[test]
    fn the_address_has_at_most_254_characters() {
        let domain = format!(
            "{}.{}.{}.com",
            "a".repeat(63),
            "b".repeat(63),
            "c".repeat(63)
        );
        let longest = format!("{}@{domain}", "x".repeat(254 - 1 - domain.len()));
        let too_long = format!("y{longest}");

        assert_eq!(longest.len(), 254);
        assert!(Email::parse(&longest).is_some());
        assert_eq!(Email::parse(&too_long), None);
    }
}
