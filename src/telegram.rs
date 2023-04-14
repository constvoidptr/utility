//! Utility to send telegram messages
//!
//! # Examples
//!
//! ```no_run
//! # use utility::telegram::Telegram;
//! let telegram = Telegram::new("<token>", "<chat_id>");
//! telegram.send_message("Hello, World!").unwrap();
//! ```

const HTTP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

pub struct Telegram {
    client: reqwest::blocking::Client,
    url_base: String,
}

impl Telegram {
    /// Create a new `Telegram` instance
    pub fn new(token: &str, chat_id: &str) -> Self {
        let client = reqwest::blocking::ClientBuilder::new()
            .https_only(true)
            .timeout(HTTP_TIMEOUT)
            .build()
            .expect("failed to build http client");
        Self::with_agent(token, chat_id, client)
    }

    /// Create a new `Telegram` instance with an already existing agent
    pub fn with_agent(token: &str, chat_id: &str, client: reqwest::blocking::Client) -> Self {
        let url_base =
            format!("https://api.telegram.org/bot{token}/sendMessage?chat_id={chat_id}&text=");

        Self { client, url_base }
    }

    /// Send a telegram message
    pub fn send_message(&self, text: &str) -> reqwest::Result<()> {
        // URL encode the message to allow for special characters
        let encoded_msg = urlencoding::encode(text);
        let url = format!("{}{}", self.url_base, encoded_msg);
        self.client.post(url).send()?;
        Ok(())
    }
}
