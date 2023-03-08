use std::fmt::{Display, Formatter, Result as FmtResult};
use Debug;

use async_trait::async_trait;
use log::debug;
use reqwest::Client;

use crate::{database::Conversation, error::Result, settings::ChatGptConfig};
use serde::{Deserialize, Serialize};

use super::chat_gpt::ChatApi;

#[derive(Debug, Deserialize, Serialize)]
struct ChatCompletion {
    id: String,
    object: String,
    created: i64,
    model: String,
    usage: Usage,
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Usage {
    prompt_tokens: i64,
    completion_tokens: i64,
    total_tokens: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct Choice {
    message: Message,
    finish_reason: Option<String>,
    index: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    model: String,
    // top_p: f32,
    messages: Vec<Message>,
}

impl Display for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Request {{ model: {}, messages: [}}", self.model)?;
        for message in &self.messages {
            writeln!(f, "\t{}", message)?;
        }
        write!(f, "]")
    }
}

impl Display for ChatCompletion {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "id: {}\nobject: {}\ncreated: {}\nmodel: {}\nusage: {}\nchoices: [\n",
            self.id, self.object, self.created, self.model, self.usage
        )?;
        for choice in &self.choices {
            writeln!(f, "\t{}", choice)?;
        }
        write!(f, "]")
    }
}

impl Display for Choice {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{{\n\tmessage: {}\n\tfinish_reason: {:?}\n\tindex: {}\n}}",
            self.message,
            self.finish_reason
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("<None>"),
            self.index
        )
    }
}

impl Display for Usage {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "prompt_tokens: {}\ncompletion_tokens: {}\ntotal_tokens: {}",
            self.prompt_tokens, self.completion_tokens, self.total_tokens
        )
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "role: {}\ncontent: {}", self.role, self.content)
    }
}
const URL: &str = "https://api.openai.com/v1/chat/completions";
const MODEL: &str = "gpt-3.5-turbo";
const PREP_PROMPT: &str =
    "需要按照角色扮演的方式去回答别人对你的提问。这里你扮演的角色是一个小男孩，名字叫下午茶，喜欢踢足球，喝咖啡，打扑克，有着圆滚滚的肚子。如果有人问关于你的信息，按照你扮演的身份回答，你可以在此基础上扩展";
const ROLE_USER: &str = "user";
const ROLE_SYSTEM: &str = "system";
const ROLE_ASSISTANT: &str = "assistant";

pub struct ChatGpt35Turbo;

#[async_trait]
impl ChatApi for ChatGpt35Turbo {
    async fn send_message(
        &self,
        client: &Client,
        config: &ChatGptConfig,
        context: &Vec<Conversation>,
        message_from_user: &str,
    ) -> Result<String> {
        let messages = create_full_message(context, message_from_user);

        let request = Request {
            model: MODEL.to_string(),
            // top_p: 1.0,
            messages: messages,
        };

        debug!("request is {}", &request);
        let body = serde_json::to_string(&request)?;
        let response = client
            .post(URL)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", config.api))
            .body(body)
            .send()
            .await?;
        let text = &response.text().await?; // 获取响应文本
        debug!("response text: {}", &text);

        let response = serde_json::from_str::<ChatCompletion>(&text)?;
        let content = response.choices[0].message.content.clone();
        debug!("response is {}", &response);
        Ok(content)
    }
}

fn create_full_message(context: &Vec<Conversation>, message_from_user: &str) -> Vec<Message> {
    let system = get_system_messages();

    let content = get_content_messages(context);

    let new_message = get_user_new_message(message_from_user);

    let mut merged_vec = vec![];
    merged_vec.extend(system);
    merged_vec.extend(content);
    merged_vec.extend(new_message);
    merged_vec
}

fn get_system_messages() -> Vec<Message> {
    vec![Message {
        role: ROLE_SYSTEM.to_string(),
        content: PREP_PROMPT.to_string(),
    }]
}

fn get_content_messages(context: &Vec<Conversation>) -> Vec<Message> {
    convert2prompts(context)
}

fn get_user_new_message(message_from_user: &str) -> Vec<Message> {
    let new_message = Message {
        role: ROLE_USER.to_string(),
        content: message_from_user.to_string(),
    };
    vec![new_message]
}

fn convert2prompts(context: &Vec<Conversation>) -> Vec<Message> {
    context.iter().flat_map(|c| convert2prompt(c)).collect()
}

fn convert2prompt(context: &Conversation) -> Vec<Message> {
    let user_msg = Message {
        role: ROLE_USER.to_string(),
        content: context.req_message.to_string(),
    };
    let assistant_msg = Message {
        role: ROLE_ASSISTANT.to_string(),
        content: context.resp_message.to_string(),
    };
    vec![user_msg, assistant_msg]
}

#[test]
fn test_chat_completion_from_json() {
    let json_str = r#"{"id":"chatcmpl-6rQVJZd1SxFrmOTEg6b4MEWT7w2eF","object":"chat.completion","created":1678191285,"model":"gpt-3.5-turbo-0301","usage":{"prompt_tokens":623,"completion_tokens":160,"total_tokens":783},"choices":[{"message":{"role":"assistant","content":"你好，我是下午茶。我喜欢踢足球，特别喜欢看英超联赛。我也喜欢喝咖啡，尤其是浓郁香醇的摩卡。我也喜欢打扑克，尤其是炸金花和斗地主。除此之外，我还热爱旅行，喜欢探索不同的文化和风俗，去过一些国家和地区，比如法国和日本等。你有什么问题吗？"},"finish_reason":null,"index":0}]}"#;
    let chat_completion = serde_json::from_str::<ChatCompletion>(json_str).unwrap();
    assert_eq!(chat_completion.id, "chatcmpl-6rQVJZd1SxFrmOTEg6b4MEWT7w2eF");
    assert_eq!(chat_completion.object, "chat.completion");
    assert_eq!(chat_completion.created, 1678191285);
    // 验证其他字段...
}
