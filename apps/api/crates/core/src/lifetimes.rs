use std::time::Duration;

pub const SESSION_ABSOLUTE_LIFETIME: Duration = Duration::from_secs(30 * 24 * 60 * 60);
pub const DISPATCH_WINDOW: Duration = Duration::from_secs(24 * 60 * 60);
pub const DISPATCH_COOLDOWN: Duration = Duration::from_secs(60);
pub const DISPATCH_LIMIT: i64 = 5;
pub const VERIFICATION_LIFETIME: Duration = Duration::from_secs(60 * 60);

const _: () = assert!(VERIFICATION_LIFETIME.as_secs().is_multiple_of(3600));

pub fn hours_text(duration: Duration) -> String {
    match duration.as_secs() / 3600 {
        1 => "1 hora".to_owned(),
        hours => format!("{hours} horas"),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::hours_text;

    #[test]
    fn whole_hours_read_in_singular_and_plural() {
        assert_eq!(hours_text(Duration::from_secs(3600)), "1 hora");
        assert_eq!(hours_text(Duration::from_secs(2 * 3600)), "2 horas");
    }
}
