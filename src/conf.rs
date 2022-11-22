use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub discord: Discord,
    pub rcon: Rcon,
    pub database: Database,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Discord {
    pub guildid: u64,
    pub token: String,
    pub guilds: Vec<String>,
    pub allowedroles: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Database {
    pub dsn: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Rcon {
    pub address: String,
    pub password: String,
}
