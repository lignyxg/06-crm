use std::fs::File;

use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub db_url: String,
    pub auth: AuthConfig,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub sk: String,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        match (
            File::open("user_stat.yml"),
            File::open("/etc/config/user_stat.yml"),
            std::env::var("USER_STAT_CONFIG"),
        ) {
            (Ok(file), _, _) => Ok(serde_yaml::from_reader(file)?),
            (_, Ok(file), _) => Ok(serde_yaml::from_reader(file)?),
            (_, _, Ok(path)) => Ok(serde_yaml::from_str(&path)?),
            _ => bail!("no config file found"),
        }
    }
}
