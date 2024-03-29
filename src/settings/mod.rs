use config::{Config, ConfigError, File};
use log::info;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub client_serial: String,
    pub proxy_cs: Option<String>,
    pub certificate_pem_path: Option<String>,
    pub certificate_root_path: Option<String>,
    // only when using native-tls
    pub certificate_key_path: Option<String>,
    pub log_level: Option<String>,
    pub esl_server_url: String,
    pub hublot_server_url: String,
    pub polling_rate: Option<i32>,
    pub pricer_user: Option<String>,
    pub pricer_password: Option<String>,
    pub parse_id: Option<String>,
    pub parse_url: Option<String>,
    pub log_file: Option<String>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let config_file = "hublot-config.toml";
        let builder = Config::builder()
            .add_source(File::with_name(config_file))
            .build()?;
        info!(
            "Settings manager have built the config from the file: {}",
            config_file
        );
        builder.try_deserialize()
    }
}
