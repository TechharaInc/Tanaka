#[macro_use]
extern crate diesel;
extern crate redis;
pub mod crud;
pub mod kvs;
pub mod models;
pub mod schema;

use diesel::{Connection, PgConnection};
use redis::{Client, RedisError};
use serde::Deserialize;
use std::{fs, io::BufReader, io::Read};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub discord_token: String,
    pub prefix: String,
    pub db_url: String,
    pub redis_url: String,
}

pub fn load_config(path: std::string::String) -> Result<Config, String> {
    let mut file_content = String::new();

    let mut fr = fs::File::open(path)
        .map(|f| BufReader::new(f))
        .map_err(|e| e.to_string())?;

    fr.read_to_string(&mut file_content)
        .map_err(|e| e.to_string())?;
    let conf: Result<Config, toml::de::Error> = toml::from_str(&file_content);
    match conf {
        Ok(p) => serde::__private::Ok(p),
        Err(e) => panic!("Filed to parse TOML: {}", e),
    }
}

pub fn establish_connection() -> PgConnection {
    let conf = match load_config("config.toml".to_string()) {
        Ok(c) => c,
        Err(e) => panic!("fail to perse toml: {}", e),
    };
    PgConnection::establish(&conf.db_url).expect(&format!("Error connecting to {}", conf.db_url))
}

pub fn open_redis_conn(path: std::string::String) -> Result<Client, RedisError> {
    redis::Client::open(path)
}
