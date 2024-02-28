//! Module for sending the mail.  
//! This defines an interface (or trait) for crate::config::Config,  
//! and implements it.  

use lettre::message::Mailbox;
use lettre::message::MessageBuilder;
use lettre::transport::smtp::client::Tls;
use lettre::transport::smtp::client::TlsParameters;
use lettre::SmtpTransport;
use lettre::Transport;

use crate::config;
use crate::info;
use crate::Args;

/// Something that can send mail when supplied with the recipient, message and subject via `args`
pub trait SendMail {
    /// Send an email to recipient specified in `args`
    fn send(&self, args: &Args) -> anyhow::Result<()>;
}

impl SendMail for config::ConfigManager {
    fn send(&self, args: &Args) -> anyhow::Result<()> {
        // Username & Decrypted Password
        let credentials = self.credentials();

        info("creating transport...");

        // The 'server' that sends the mail via SMTP
        let mailer = SmtpTransport::relay(&self.config.relay_settings.addr)?
            .authentication(self.config.relay_settings.authentication.clone())
            .port(self.config.relay_settings.port)
            .tls(
                if self.config.relay_settings.tls {
                    Tls::Required( TlsParameters::new(self.config.relay_settings.addr.to_string())? )
                } else {
                    Tls::None
                }
            )
            .credentials(credentials)
            .build();

        //NOTE: maybe find a way around the cloning.
        info("building message...");
        let message = MessageBuilder::new()
            .from(Mailbox::new(None, self.username().parse().unwrap()))
            .to(args.to.parse().map_err(|err| anyhow::anyhow!("failed to parse --to '{}': {err}", args.to))?)
            .subject(args.subject.clone())
            .body(args.msg.clone())?;

        info("sending message...");
        Ok(mailer.send(&message).map(|_| ())?) // Disregard Ok(value)
    }
}