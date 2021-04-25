// use std::env;

extern crate serde;
extern crate toml;
use crate::kvs::Database;
use serde::Deserialize;
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        channel::Message,
        prelude::{ReactionType, Ready},
    },
    prelude::Client,
};
use std::io::Read;
use std::{fs, io::BufReader};

struct Handler;

#[derive(Debug, Deserialize)]
struct Config {
    db_path: String,
    discord_token: String,
}

mod consts;
use crate::consts::consts::REACTION_FAILED;

mod kvs;

fn load_config(path: std::string::String) -> Result<Config, String> {
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

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let data = ctx.data.read().await;
        let db = data.get::<Database>().unwrap();
        match db.put(b"Hello", b"World") {
            Ok(v) => v,
            Err(e) => print!("{}", e),
        };
        if !msg.author.bot {
            if let Ok(Some(v)) = db.get(msg.content.as_bytes()) {
                if v.len() != 0 {
                    if let Err(why) = msg
                        .channel_id
                        .say(&ctx.http, String::from_utf8(v).unwrap())
                        .await
                    {
                        if let Err(why) = msg
                            .react(
                                &ctx.http,
                                ReactionType::Unicode(REACTION_FAILED.to_string()),
                            )
                            .await
                        {
                            println!("Error reacting message: {:?}", why);
                        };
                        println!("Error sending message: {:?}", why);
                    }
                }
            }
        };
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let conf = match load_config("config.toml".to_string()) {
        Ok(c) => c,
        Err(e) => panic!("fail to perse toml: {}", e),
    };

    let db: kvs::RocksDB = kvs::KVStore::init(&conf.db_path);

    let mut client = Client::builder(&conf.discord_token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    let data = client.data.write();
    data.await.insert::<Database>(db.db.clone());

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
