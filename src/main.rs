use std::{sync::Arc, time::Duration};

use actix_web::{web::Data, App, HttpServer};
use log::info;

use reqwest::Client;
use simple_logger::SimpleLogger;
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlSslMode},
    MySql, Pool,
};

use crate::{
    cache::Cache,
    error::Result,
    handlers::{handle_wechat_message, index},
    settings::{ChatGptConfig, Database, Settings, WechatConfig},
};

mod api;
mod cache;
mod database;
mod error;
mod handlers;
mod settings;

#[derive(Clone)]
struct AppState {
    pool: Pool<MySql>,
    client: Client,
    chat_gpt_config: ChatGptConfig,
    wechat_config: WechatConfig,
    cache: Arc<Cache>,
}

async fn get_pool(database: Database) -> Result<Pool<MySql>> {
    let options = MySqlConnectOptions::new()
        .host(&database.host)
        .username(&database.username)
        .password(&database.password)
        .database(&database.database)
        .ssl_mode(MySqlSslMode::Preferred)
        .ssl_ca(&std::path::PathBuf::from(&database.ssl_ca_path()));

    let pool = MySqlPoolOptions::new()
        .max_connections(database.max_connections)
        .connect_with(options)
        .await?;
    Ok(pool)
}

#[actix_web::main]
async fn main() -> Result<()> {
    let s = Settings::new().unwrap();

    SimpleLogger::new()
        .with_level(s.log.get_level_filter())
        .init()
        .unwrap();

    let cache = Arc::new(Cache::new());
    let cache_clone = Arc::clone(&cache);
    let ttl = Duration::from_secs(60);
    tokio::spawn(async move {
        cache_clone.cleanup(ttl).await;
    });

    let pool = get_pool(s.database).await?;

    let client = Client::new();

    let chat_gpt_config = s.chat_gpt_config;

    let wechat_config = s.wechat_config;

    let app_state = AppState {
        pool,
        client,
        chat_gpt_config,
        wechat_config,
        cache: Arc::clone(&cache),
    };

    let ip = s.server.get_ip();

    info!("server listening at http://{:?}", &ip);
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_state.clone()))
            .service(handle_wechat_message)
            .service(index)
    })
    .bind(&ip)?
    .run()
    .await?;

    Ok(())
}
