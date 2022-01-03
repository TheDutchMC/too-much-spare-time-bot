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
    pub token: String,
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
    database: String,
    username: String,
    password: String,
}

impl Config {
    pub fn new(path: &Path) -> Result<Self> {
        let f = fs::File::open(path)?;
        let config: Self = serde_yaml::from_reader(&f)?;
        Ok(config)
    }

    pub fn mysql_connection(&self) -> Result<Pool> {
        let opts = OptsBuilder::new()
            .ip_or_hostname(Some(&self.mysql.host))
            .pass(Some(&self.mysql.password))
            .user(Some(&self.mysql.username))
            .db_name(Some(&self.mysql.database));

        let pool = Pool::new(opts)?;
        Ok(pool)
    }
}
