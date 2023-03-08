use config::{Config, ConfigError};
use log::LevelFilter;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub log: Log,
    pub server: Server,
    pub wechat_config: WechatConfig,
    pub database: Database,
    pub chat_gpt_config: ChatGptConfig,
}

#[derive(Debug, Deserialize)]
pub struct Log {
    pub level: String,
}

impl Log {
    // Set the maximum log level based on the configuration file.
    pub fn get_level_filter(&self) -> LevelFilter {
        match self.level.as_str() {
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => LevelFilter::Info,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub port: u32,
    pub ip: String,
}

impl Server {
    pub fn get_ip(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Deserialize)]
pub struct Database {
    // pub url: String,
    pub host: String,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
    pub ssl_ca_name: String,
}

impl Database {
    pub fn ssl_ca_path(&self) -> String {
        format!("{}{}", CURRENT_DIR, self.ssl_ca_name)
    }
}
#[derive(Debug, Deserialize, Clone)]
pub struct WechatConfig {
    pub app_id: String,
    pub app_secret: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChatGptConfig {
    pub api: String,
    pub model: String,
}

const CURRENT_DIR: &str = "./config/";
const SEETING_NAME: &str = "Settings.toml";

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(config::File::with_name(
                format!("{}{}", CURRENT_DIR, SEETING_NAME).as_str(),
            ))
            .build()?;

        s.try_deserialize()
    }
}
