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

#[cfg(test)]
mod tests {
    use crate::api::chat_gpt_35_turbo::ChatGpt35Turbo;
    use crate::{settings::ChatGptConfig};

    use super::*;

    #[tokio::test]
    async fn test_send_message() {
        let api = ChatGpt35Turbo {};

        let client = Client::new();
        let config = ChatGptConfig {
            api: "".to_owned(),
            model: "gpt-3.5-turbo".to_owned(),
        };
        let context = vec![];
        let message_from_user = "Hi, there!";

        let result = api
            .send_message(&client, &config, &context, message_from_user)
            .await;

        // Check if the result is a string
        assert!(result.is_ok());
    }
}
