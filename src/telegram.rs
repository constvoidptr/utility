//! Utility to send telegram messages
//!
//! # Examples
//!
//! ```no_run
//! # use utility::telegram::Telegram;
//! let telegram = Telegram::new("<token>", "<chat_id>");
//! telegram.send_message("Hello, World!").unwrap();
//! ```

use std::sync::Arc;

pub struct Telegram {
    agent: ureq::Agent,
    url_base: String,
}

impl Telegram {
    /// Create a new `Telegram` instance
    pub fn new(token: &str, chat_id: &str) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(3))
            .tls_connector(Arc::new(
                native_tls::TlsConnector::new().expect("failed to create tls connector"),
            ))
            .build();

        Self::with_agent(token, chat_id, agent)
    }

    /// Create a new `Telegram` instance with an already existing agent
    pub fn with_agent(token: &str, chat_id: &str, agent: ureq::Agent) -> Self {
        let url_base =
            format!("https://api.telegram.org/bot{token}/sendMessage?chat_id={chat_id}&text=");

        Self { agent, url_base }
    }

    /// Send a telegram message
    pub fn send_message(&self, text: &str) -> Result<(), Box<ureq::Error>> {
        // URL encode the message to allow for special characters
        let encoded_msg = urlencoding::encode(text);
        let url = format!("{}{}", self.url_base, encoded_msg);
        self.agent.post(&url).call().map_err(Box::new)?;
        Ok(())
    }
}
