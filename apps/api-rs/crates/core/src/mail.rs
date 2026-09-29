use std::error::Error;

use askama::Template;
use lettre::message::{Mailbox, MultiPart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::stub::AsyncStubTransport;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::config::Config;

type MailError = Box<dyn Error + Send + Sync>;

#[derive(Clone)]
enum Transport {
    Smtp(AsyncSmtpTransport<Tokio1Executor>),
    Stub(AsyncStubTransport),
}

#[derive(Clone)]
pub struct Mailer {
    transport: Transport,
    from: Mailbox,
}

pub struct MailMessage {
    subject: &'static str,
    html: String,
    text: String,
}

impl Mailer {
    pub fn smtp(config: &Config) -> Result<Self, MailError> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host)?
            .port(config.smtp_port)
            .credentials(Credentials::new(
                config.smtp_user.clone(),
                config.smtp_password.clone(),
            ))
            .build();
        Self::with(config, Transport::Smtp(transport))
    }

    pub fn stub(config: &Config, stub: AsyncStubTransport) -> Result<Self, MailError> {
        Self::with(config, Transport::Stub(stub))
    }

    fn with(config: &Config, transport: Transport) -> Result<Self, MailError> {
        let from = Mailbox::new(Some("Clinicore".to_string()), config.mail_from.parse()?);
        Ok(Self { transport, from })
    }

    pub fn send(&self, to: &str, message: MailMessage) {
        let mailer = self.clone();
        let to = to.to_string();
        tokio::spawn(async move {
            if let Err(reason) = mailer.deliver(&to, message).await {
                tracing::error!(email = %to, reason = %reason, "Mail delivery failed");
            }
        });
    }

    async fn deliver(&self, to: &str, message: MailMessage) -> Result<(), MailError> {
        let email = Message::builder()
            .from(self.from.clone())
            .to(to.parse()?)
            .subject(message.subject)
            .multipart(MultiPart::alternative_plain_html(
                message.text,
                message.html,
            ))?;
        match &self.transport {
            Transport::Smtp(smtp) => {
                smtp.send(email).await?;
            }
            Transport::Stub(stub) => {
                stub.send(email).await?;
            }
        }
        Ok(())
    }
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
