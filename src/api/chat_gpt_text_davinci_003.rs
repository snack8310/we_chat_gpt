use std::fmt::{self, Debug};

use async_trait::async_trait;
use log::{debug, info};
use reqwest::Client;

use crate::{database::Conversation, error::Result, settings::ChatGptConfig};
use serde::{Deserialize, Serialize};

use super::chat_gpt::ChatApi;

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    text: String,
    index: usize,
    logprobs: Option<serde_json::Value>,
    finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatGptResponse {
    id: String,
    object: String,
    created: usize,
    model: String,
    choices: Vec<Choice>,
    usage: serde_json::Value,
}

impl fmt::Display for ChatGptResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = format!("ChatGptResponse:\n");
        s += &format!("  id: {}\n", self.id);
        s += &format!("  object: {}\n", self.object);
        s += &format!("  created: {}\n", self.created);
        s += &format!("  model: {}\n", self.model);
        s += &format!("  choices:\n");

        for choice in &self.choices {
            s += &format!("    {}\n", choice.text);
        }

        s += &format!("  usage: {}\n", self.usage);

        write!(f, "{}", s)
    }
}

const PREP_PROMPT: &str =
"假装你是一个小男孩，名字叫下午茶，喜欢踢足球，喝咖啡，打扑克，有着圆滚滚的肚子。后面问到所有你相关的信息，请你都以这个角色角度回答";
const MODEL: &str = "text-davinci-003";
const QUESTION_MARK: &str = "SNACKQQQQ:";
const ANSWER_MARK: &str = "SNACKAAAA:";
const URL: &str = "https://api.openai.com/v1/chat/completions";

pub struct ChatGptTextDavinci003;

#[async_trait]
impl ChatApi for ChatGptTextDavinci003 {
    async fn send_message(
        &self,
        client: &Client,
        config: &ChatGptConfig,
        context: &Vec<Conversation>,
        message_from_user: &str,
    ) -> Result<String> {
        debug!(
            "send_message with context: {} message form user: {}",
            convert2prompts(context),
            message_from_user
        );

        let prompt = format!(
            "{}{}{}{}{}",
            PREP_PROMPT,
            convert2prompts(context),
            QUESTION_MARK,
            message_from_user,
            ANSWER_MARK
        );
        info!("prompt is {}", prompt.clone());
        let request_body = format!(
            r#"{{"model": "{}", "prompt": "{}", "temperature": 0, "max_tokens": 100,"stop": [
            "{}",
            "{}"
        ]}}"#,
            MODEL, prompt, QUESTION_MARK, ANSWER_MARK
        );

        let response = client
            .post(URL)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", config.api))
            .body(request_body)
            .send()
            .await?
            .json::<ChatGptResponse>()
            .await?;
        let text = response.choices[0].text.clone();
        info!("response is {}", &response);
        Ok(text)
    }
}

fn convert2prompts(context: &Vec<Conversation>) -> String {
    context.iter().map(|c| convert2prompt(c)).collect()
}

fn convert2prompt(context: &Conversation) -> String {
    format!(
        "{}{}{}{}",
        QUESTION_MARK, context.req_message, ANSWER_MARK, context.resp_message
    )
}
