use lettre::message::Mailbox;
use lettre::message::MessageBuilder;
use lettre::transport::smtp::client::Tls;
use lettre::transport::smtp::client::TlsParameters;
use lettre::SmtpTransport;
use lettre::Transport;

use crate::config;
use crate::Args;

pub trait SendMail {
    fn send(&self, args: &Args) -> anyhow::Result<()>;
}

impl SendMail for config::Config {
    fn send(&self, args: &Args) -> anyhow::Result<()> {
        // Username & Decrypted Password
        let credentials = self.credentials();

        println!("creating transport...");
        let mailer = SmtpTransport::relay(&self.relay_settings.addr)?
            .authentication(self.relay_settings.authentication.clone())
            .port(self.relay_settings.port)
            .tls(
                if self.relay_settings.tls {
                    Tls::Required( TlsParameters::new(self.relay_settings.addr.to_string())? )
                } else {
                    Tls::None
                }
            )
            .credentials(credentials)
            .build();

        println!("building message...");
        let message = MessageBuilder::new()
            .from(Mailbox::new(None, self.username().parse().unwrap()))
            .to(args.to.parse().unwrap())
            .subject(args.subject.clone())
            .body(args.msg.clone())?;

        println!("sending message...");
        Ok(mailer.send(&message).map(|_| ())?)
    }
}