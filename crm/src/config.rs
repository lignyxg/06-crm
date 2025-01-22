use std::fs::File;

use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub db_url: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub metadata: String,
    pub user_stat: String,
    pub notification: String,
    pub auth: String,
    pub tls: Option<TlsConfig>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub cert: String,
    pub key: String,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        match (
            File::open("crm.yml"),
            File::open("/etc/config/crm.yml"),
            std::env::var("CRM_CONFIG"),
        ) {
            (Ok(file), _, _) => Ok(serde_yaml::from_reader(file)?),
            (_, Ok(file), _) => Ok(serde_yaml::from_reader(file)?),
            (_, _, Ok(path)) => Ok(serde_yaml::from_str(&path)?),
            _ => bail!("no config file found"),
        }
    }
}
