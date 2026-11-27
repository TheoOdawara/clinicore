use std::time::Duration;

use clinicore_core::config::{SmtpTls, WorkerConfig};
use lettre::message::{Mailbox, MultiPart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

const SEND_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, thiserror::Error)]
pub enum MailError {
    #[error("the address is not a mailbox: {0}")]
    Address(#[from] lettre::address::AddressError),
    #[error("the message could not be built: {0}")]
    Message(#[from] lettre::error::Error),
    #[error("the SMTP server did not take the message: {0}")]
    Smtp(#[from] lettre::transport::smtp::Error),
}

pub struct Outgoing<'a> {
    pub to: &'a str,
    pub subject: &'a str,
    pub html: &'a str,
    pub text: &'a str,
}

pub struct Mailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
    redirect_to: Option<String>,
}

impl Mailer {
    pub fn new(config: &WorkerConfig) -> Result<Self, MailError> {
        let builder = match config.smtp_tls {
            SmtpTls::Starttls => {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host)?
            }
            SmtpTls::Tls => AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)?,
            SmtpTls::None => {
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.smtp_host)
            }
        };
        let transport = builder
            .port(config.smtp_port)
            .credentials(Credentials::new(
                config.smtp_user.clone(),
                config.smtp_password.clone(),
            ))
            .timeout(Some(SEND_TIMEOUT))
            .build();
        Ok(Self {
            transport,
            from: config.mail_from.parse()?,
            redirect_to: config.mail_redirect_to.clone(),
        })
    }

    pub async fn send(&self, mail: &Outgoing<'_>) -> Result<(), MailError> {
        let to: Mailbox = self.redirect_to.as_deref().unwrap_or(mail.to).parse()?;
        let message = Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject(mail.subject)
            .multipart(MultiPart::alternative_plain_html(
                mail.text.to_owned(),
                mail.html.to_owned(),
            ))?;
        self.transport.send(message).await?;
        Ok(())
    }
}
