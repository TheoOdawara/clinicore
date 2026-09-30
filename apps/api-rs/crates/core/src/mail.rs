use askama::Template;
use lettre::message::{Mailbox, MultiPart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::stub::AsyncStubTransport;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum MailError {
    #[error(transparent)]
    Address(#[from] lettre::address::AddressError),
    #[error(transparent)]
    Message(#[from] lettre::error::Error),
    #[error(transparent)]
    MessageId(#[from] getrandom::Error),
    #[error(transparent)]
    Smtp(#[from] lettre::transport::smtp::Error),
    #[error(transparent)]
    Stub(#[from] lettre::transport::stub::Error),
    #[error(transparent)]
    Template(#[from] askama::Error),
}

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

    // ponytail: fire-and-forget spawn, a restart or a transient SMTP failure loses the mail; outbox table and the worker retry in #71
    pub fn send(&self, to: &str, message: MailMessage) {
        let mailer = self.clone();
        let to = to.to_string();
        tokio::spawn(async move {
            if let Err(reason) = mailer.deliver(&to, message).await {
                tracing::error!(email = %masked(&to), reason = %reason, "Mail delivery failed");
            }
        });
    }

    async fn deliver(&self, to: &str, message: MailMessage) -> Result<(), MailError> {
        let mut unique = [0u8; 16];
        getrandom::fill(&mut unique)?;
        let email = Message::builder()
            .message_id(Some(format!(
                "<{}@{}>",
                hex::encode(unique),
                self.from.email.domain()
            )))
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

fn masked(address: &str) -> String {
    let Some((local, domain)) = address.split_once('@') else {
        return "***".to_string();
    };
    let first: String = local.chars().take(1).collect();
    format!("{first}***@{domain}")
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

pub fn compose(subject: &'static str, content: &MailContent) -> Result<MailMessage, MailError> {
    Ok(MailMessage {
        subject,
        html: HtmlBody { content }.render()?,
        text: TextBody { content }.render()?,
    })
}
