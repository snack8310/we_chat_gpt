use std::time::Duration;

use log::debug;
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};

use crate::error::Result;

pub async fn save_conversation(
    pool: &Pool<MySql>,
    msg_id: i64,
    user_id: &str,
    subscription_id: &str,
    message_from_user: &str,
    message_from_chat: &str,
    elapsed: Duration,
) -> Result<()> {
    debug!("save_conversation begin");
    let sql = "INSERT INTO wechat_dialogue_record(msg_id, user_id, subscription_id, type_id, message, elapsed, created_time) VALUES (?, ?, ?, ?, ?, ?, NOW())";
    let _result = sqlx::query(sql)
        .bind(msg_id)
        .bind(user_id)
        .bind(subscription_id)
        .bind("message")
        .bind(merge2conversation(message_from_user, message_from_chat))
        .bind(elapsed.as_millis() as i64)
        .fetch_all(pool)
        .await?;

    debug!("save_conversation end");
    Ok(())
}

pub async fn get_conversation_by_msg_id(pool: &Pool<MySql>, msg_id: i64) -> Result<String> {
    let row: (String,) =
        sqlx::query_as("SELECT message FROM wechat_dialogue_record WHERE msg_id = ?")
            .bind(msg_id)
            .fetch_one(pool)
            .await?;
    let msg: Conversation = serde_json::from_str(row.0.as_str()).unwrap();
    Ok(msg.resp_message)
}

pub async fn get_conversations(
    pool: &Pool<MySql>,
    user_id: &str,
    subscription_id: &str,
) -> Result<Vec<Conversation>> {
    let rows: Vec<(String,)> = sqlx::query_as("SELECT message FROM wechat_dialogue_record WHERE user_id = ? AND subscription_id = ? ORDER BY created_time DESC LIMIT ?")
        .bind(user_id).bind(subscription_id).bind(LIMIT_COUNT)
        .fetch_all(pool)
        .await?;
    let conversations: Vec<Conversation> = rows
        .iter()
        .map(|row| serde_json::from_str(row.0.as_str()).unwrap())
        .collect();

    //最后n条记录-1，-2，-3，-4，...， -n 需要正序排列，-n, ... -4, -3, -2, -1
    Ok(conversations.iter().rev().cloned().collect())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Conversation {
    pub req_message: String,
    pub resp_message: String,
}

fn merge2conversation(message_from_user: &str, message_from_chat: &str) -> String {
    let msg = Conversation {
        req_message: message_from_user.to_string(),
        resp_message: message_from_chat.to_string(),
    };
    serde_json::to_string(&msg).unwrap()
}

const LIMIT_COUNT: u8 = 10;
