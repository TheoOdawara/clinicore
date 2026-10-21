use askama::Template;

pub struct MailMessage {
    pub subject: &'static str,
    pub html: String,
    pub text: String,
}

pub struct MailContent<'a> {
    pub name: &'a str,
    pub paragraphs: &'a [&'a str],
    pub action: &'a str,
    pub link: &'a str,
}

#[derive(Template)]
#[template(path = "mail.html")]
struct HtmlBody<'a> {
    content: &'a MailContent<'a>,
}

#[derive(Template)]
#[template(path = "mail.txt")]
struct TextBody<'a> {
    content: &'a MailContent<'a>,
}

pub fn compose(subject: &'static str, content: &MailContent) -> Result<MailMessage, askama::Error> {
    Ok(MailMessage {
        subject,
        html: HtmlBody { content }.render()?,
        text: TextBody { content }.render()?,
    })
}
