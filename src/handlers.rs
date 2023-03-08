use std::time::{Duration, Instant};

use actix_web::{get, post, web, HttpResponse};
use log::{debug, warn};

use actix_xml::Xml;
use serde::Deserialize;
use serde_xml_rs::to_string;

use crate::{
    api::{
        chat_gpt::ChatApi,
        chat_gpt_35_turbo::ChatGpt35Turbo,
        chat_gpt_text_davinci_003::ChatGptTextDavinci003,
        wechat::{verify_signature, TextMessage, WeChatRequest},
    },
    database::{get_conversation_by_msg_id, get_conversations, save_conversation},
    error::Result,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct WeChatMessage {
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
    #[serde(rename = "MsgId")]
    pub msg_id: i64,
}

#[post("/")]
async fn handle_wechat_message(
    wechat_message: Xml<WeChatMessage>,
    data: web::Data<AppState>,
) -> Result<HttpResponse> {
    let start = Instant::now();
    let app_state = data.get_ref();

    debug!("received wechat message: {:?}", &wechat_message);

    let msg_id = wechat_message.msg_id;
    let user_id = wechat_message.from_user_name.clone();
    let subscription_id = wechat_message.to_user_name.clone();

    let cache = &data.cache;
    let key = format!("WECHAT_MSG_ID_{}", msg_id.to_string());
    match cache.get(&key).await {
        Some(_i) => {
            warn!("there is re_call from wechat, key is {:?}", &key);
            let message_from_cache = get_conversation_by_msg_id(&app_state.pool, msg_id).await?;
            warn!("the re_call get correct result, key is {:?}", &key);
            let xml_response = get_response_xml(user_id, subscription_id, message_from_cache);

            return Ok(HttpResponse::Ok()
                .content_type("text/xml")
                .body(xml_response));
        }
        None => {
            let value = std::time::Instant::now();
            let ttl = Duration::from_secs(60);
            cache.set(&key, value, ttl).await;
        }
    }

    let context = get_conversations(&app_state.pool, &user_id, &subscription_id).await?;

    debug!("send prompt to chatgpt");

    let api: Box<dyn ChatApi> = match app_state.chat_gpt_config.model.as_str() {
        "text-davinci-003" => Box::new(ChatGptTextDavinci003),
        "gpt-3.5-turbo" => Box::new(ChatGpt35Turbo),
        _ => return Ok(HttpResponse::BadRequest().finish()),
    };
    let message_from_chat = api
        .send_message(
            &app_state.client,
            &app_state.chat_gpt_config,
            &context,
            &wechat_message.content,
        )
        .await?;

    debug!("get result to chatgpt");

    let elapsed = start.elapsed();

    save_conversation(
        &app_state.pool,
        msg_id,
        &user_id,
        &subscription_id,
        &wechat_message.content,
        &message_from_chat,
        elapsed,
    )
    .await?;

    let xml_response = get_response_xml(user_id, subscription_id, message_from_chat);

    Ok(HttpResponse::Ok()
        .content_type("text/xml")
        .body(xml_response))
}

fn get_response_xml(to_user_name: String, from_user_name: String, content: String) -> String {
    let text_message = TextMessage::new(to_user_name, from_user_name, content);

    debug!("xml_response: {:?}", &text_message);
    to_string(&text_message).unwrap()
}

#[get("/")]
async fn index(info: web::Query<WeChatRequest>, data: web::Data<AppState>) -> Result<HttpResponse> {
    debug!("info is {:?}", &info);
    let echostr = info.echostr.clone();
    let app_state = data.get_ref();
    verify_signature(&info, &app_state.wechat_config.token)?;

    let echostr = match echostr {
        Some(s) => s,
        None => return Ok(HttpResponse::BadRequest().finish()),
    };
    Ok(HttpResponse::Ok().body(echostr))
}
