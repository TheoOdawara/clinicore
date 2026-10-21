use std::time::Duration;

use clinicore_core::config::WorkerConfig;
use lettre::message::{Mailbox, MultiPart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::stub::AsyncStubTransport;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum MailError {
    #[error(transparent)]
    Address(#[from] lettre::address::AddressError),
    #[error(transparent)]
    Message(#[from] lettre::error::Error),
    #[error(transparent)]
    Smtp(#[from] lettre::transport::smtp::Error),
    #[error(transparent)]
    Stub(#[from] lettre::transport::stub::Error),
}

pub enum Transport {
    Smtp(AsyncSmtpTransport<Tokio1Executor>),
    Stub(AsyncStubTransport),
}

pub struct Mailer {
    transport: Transport,
    from: Mailbox,
}

impl Mailer {
    pub fn smtp(config: &WorkerConfig) -> Result<Self, MailError> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host)?
            .port(config.smtp_port)
            .credentials(Credentials::new(
                config.smtp_user.clone(),
                config.smtp_password.clone(),
            ))
            .timeout(Some(Duration::from_secs(10)))
            .build();
        Self::new(&config.mail_from, Transport::Smtp(transport))
    }

    pub fn new(mail_from: &str, transport: Transport) -> Result<Self, MailError> {
        let from = Mailbox::new(Some("Clinicore".to_string()), mail_from.parse()?);
        Ok(Self { transport, from })
    }

    pub async fn deliver(
        &self,
        to: &str,
        subject: String,
        html: String,
        text: String,
    ) -> Result<(), MailError> {
        let email = Message::builder()
            .message_id(Some(format!(
                "<{}@{}>",
                Uuid::new_v4(),
                self.from.email.domain()
            )))
            .from(self.from.clone())
            .to(to.parse()?)
            .subject(subject)
            .multipart(MultiPart::alternative_plain_html(text, html))?;
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
