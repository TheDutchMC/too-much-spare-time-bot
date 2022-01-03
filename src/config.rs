use anyhow::Result;
use mysql::{OptsBuilder, Pool};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub discord: DiscordConfig,
    mysql: MysqlConfig,
    pub roles: Vec<RoleConfig>,
}

#[derive(Deserialize, Clone)]
pub struct DiscordConfig {
    pub token: Option<String>,
    pub prefix: String,
}

#[derive(Deserialize, Clone)]
pub struct RoleConfig {
    pub id: u64,
    pub messages: u64,
}

#[derive(Deserialize, Clone)]
pub struct MysqlConfig {
    host: String,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

impl Config {
    pub fn new(path: &Path) -> Result<Self> {
        let f = fs::File::open(path)?;
        let mut config: Self = serde_yaml::from_reader(&f)?;

        if config.discord.token.is_none() {
            config.discord.token = Some(std::env::var("TOKEN").expect("Missing Discord token"));
        }

        if config.mysql.username.is_none() {
            config.mysql.username = Some(std::env::var("MYSQL_USERNAME").expect("Missing MySQL username"));
        }

        if config.mysql.database.is_none() {
            config.mysql.database = Some(std::env::var("MYSQL_DATABASE").expect("Missing MySQL database"));
        }

        if config.mysql.password.is_none() {
            config.mysql.password = Some(std::env::var("MYSQL_PASSWORD").expect("Missing MySQL password"));
        }

        Ok(config)
    }

    pub fn mysql_connection(&self) -> Result<Pool> {
        let opts = OptsBuilder::new()
            .ip_or_hostname(Some(&self.mysql.host))
            .pass(Some(self.mysql.password.as_ref().unwrap()))
            .user(Some(self.mysql.username.as_ref().unwrap()))
            .db_name(Some(self.mysql.database.as_ref().unwrap()));

        let pool = Pool::new(opts)?;
        Ok(pool)
    }
}
