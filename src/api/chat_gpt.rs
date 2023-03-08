use async_trait::async_trait;
use reqwest::Client;

use crate::{database::Conversation, error::Result, settings::ChatGptConfig};

#[async_trait]
pub trait ChatApi {
    async fn send_message(
        &self,
        client: &Client,
        config: &ChatGptConfig,
        context: &Vec<Conversation>,
        message_from_user: &str,
    ) -> Result<String>;
}
