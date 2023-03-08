use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use sha1::{Digest, Sha1};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Debug)]
#[serde(rename = "xml")]
pub struct TextMessage {
    #[serde(rename = "ToUserName")]
    pub to_user_name: String,
    #[serde(rename = "FromUserName")]
    pub from_user_name: String,
    #[serde(rename = "CreateTime")]
    pub create_time: i64,
    #[serde(rename = "MsgType")]
    pub msg_type: String,
    #[serde(rename = "Content")]
    pub content: String,
}

impl TextMessage {
    pub fn new(to_user_name: String, from_user_name: String, content: String) -> Self {
        Self {
            to_user_name,
            from_user_name,
            create_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            msg_type: "text".to_owned(),
            content,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct WeChatRequest {
    #[serde(rename = "signature")]
    pub signature: String,
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "nonce")]
    pub nonce: String,
    #[serde(rename = "echostr")]
    pub echostr: Option<String>,
    #[serde(flatten)]
    pub message: Option<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "ToUserName")]
    to_user_name: String,
    #[serde(rename = "FromUserName")]
    from_user_name: String,
    #[serde(rename = "CreateTime")]
    create_time: i64,
    #[serde(rename = "MsgType")]
    msg_type: String,
    #[serde(rename = "Content")]
    content: Option<String>,
    #[serde(rename = "MsgId")]
    msg_id: Option<i64>,
}

pub fn verify_signature(info: &WeChatRequest, token: &str) -> Result<()> {
    let mut values = vec![token, &info.timestamp, &info.nonce];
    values.sort();
    let input = values.join("");
    let mut hasher = Sha1::new();
    hasher.update(input.as_bytes());
    let result = hex::encode(hasher.finalize());

    if result != *&info.signature {
        return Err(Error::InvalidSignature);
    }

    Ok(())
}
