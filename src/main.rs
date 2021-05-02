use rand::prelude::SliceRandom;
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::standard::{
        macros::{command, group, hook},
        Args, CommandResult, StandardFramework,
    },
    http::Http,
    model::{channel::Message, gateway::Ready, prelude::ReactionType},
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use serenity::prelude::*;
use tokio::sync::Mutex;

use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};

struct DbConn;

impl TypeMapKey for DbConn {
    type Value = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>;
}

struct RedisConn;

impl TypeMapKey for RedisConn {
    type Value = redis::Client;
}

mod consts;

use crate::consts::consts::{REACTION_FAILED, REACTION_SUCESSED};

mod crud;

use kwbot::*;
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(add, remove, rank, alias)]
struct Resp;

#[command("alias")]
#[sub_commands(add_alias, remove_alias)]
async fn alias(_: &Context, _: &Message, _args: Args) -> CommandResult {
    Ok(())
}

#[command]
async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if msg.guild_id == None {
        if let Err(why) = msg
            .channel_id
            .say(&ctx.http, "このコマンドは DM で実行できません")
            .await
        {
            println!("Error sending message: {:?}", why);
        }
    };

    if args.len() < 2 && msg.attachments.is_empty() {
        if let Err(why) = msg.channel_id.say(&ctx.http, "引数が足りません").await {
            println!("Error sending message: {:?}", why);
        }
    } else {
        let data = ctx.data.read().await;
        let db = data.get::<DbConn>().unwrap().clone();
        let conn = db.get().unwrap();

        let key = args.single::<String>().unwrap();
        let value = if msg.attachments.is_empty() {
            args.rest()
        } else {
            &msg.attachments[0].url
        };

        let emoji = match crud::add_command(
            conn,
            msg.guild_id.unwrap().to_string(),
            key,
            value.to_string(),
            format!("{}", msg.author.id),
        ) {
            Ok(_) => REACTION_SUCESSED,
            Err(why) => {
                println!("{}", why);
                REACTION_FAILED
            }
        };

        if let Err(why) = msg
            .react(&ctx.http, ReactionType::Unicode(emoji.to_string()))
            .await
        {
            println!("Error reacting message: {:?}", why);
        };
    }
    Ok(())
}

#[command]
async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() || args.len() < 1 {
        if let Err(why) = msg.channel_id.say(&ctx.http, "引数が足りません").await {
            println!("Error sending message: {:?}", why);
        }
    } else {
        let data = ctx.data.read().await;
        let db = data.get::<DbConn>().unwrap().clone();
        let kvs_conn = data.get::<RedisConn>().unwrap().clone();
        let conn = db.get().unwrap();

        let gid: String = msg.guild_id.unwrap().to_string();
        let key: String = args.single::<String>().unwrap();

        kvs::command_delete(
            &mut kvs_conn.get_connection().unwrap(),
            gid.clone(),
            key.clone(),
        );

        let emoji = match crud::command_delete(conn, gid.clone(), key.clone()) {
            Ok(_) => REACTION_SUCESSED,
            Err(_) => REACTION_FAILED,
        };

        if let Err(why) = msg
            .react(&ctx.http, ReactionType::Unicode(emoji.to_string()))
            .await
        {
            println!("Error reacting message: {:?}", why);
        };
    }
    Ok(())
}

#[command]
async fn rank(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let kvs_conn = data.get::<RedisConn>().unwrap().clone();

    let gid: String = msg.guild_id.unwrap().to_string();

    let mut rank = vec![];
    for (i, r) in kvs::command_rank(&mut kvs_conn.get_connection().unwrap(), gid.clone())
        .iter()
        .enumerate()
    {
        rank.push(format!("{}位 {} ({}回)", i + 1, r.0, r.1));
    }

    if let Err(why) = msg.channel_id.say(&ctx.http, rank.join("\n")).await {
        println!("Error sending message: {:?}", why);
    };
    Ok(())
}

#[command]
#[aliases("add")]
async fn add_alias(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let key = args.single::<String>().unwrap();
    let value = args.rest();

    let data = ctx.data.read().await;
    let kvs_conn = data.get::<RedisConn>().unwrap().clone();

    let gid: String = msg.guild_id.unwrap().to_string();

    if args.len() < 2 {
        if let Err(why) = msg.channel_id.say(&ctx.http, "引数が足りません").await {
            println!("Error sending message: {:?}", why);
        }
    } else {
        match kvs::add_alias(
            &mut kvs_conn.get_connection().unwrap(),
            gid,
            value.to_string(),
            key,
        ) {
            Ok(_) => {
                if let Err(why) = msg
                    .react(
                        &ctx.http,
                        ReactionType::Unicode(REACTION_SUCESSED.to_string()),
                    )
                    .await
                {
                    println!("Error reacting message: {:?}", why);
                };
            }
            Err(why) => {
                println!("Error adding alias: {:?}", why);
                if let Err(why) = msg
                    .react(
                        &ctx.http,
                        ReactionType::Unicode(REACTION_FAILED.to_string()),
                    )
                    .await
                {
                    println!("Error reacting message: {:?}", why);
                };
            }
        }
    }
    Ok(())
}

#[command]
#[aliases("remove")]
async fn remove_alias(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let key = args.single::<String>().unwrap();

    let data = ctx.data.read().await;
    let kvs_conn = data.get::<RedisConn>().unwrap().clone();

    let gid: String = msg.guild_id.unwrap().to_string();

    match kvs::remove_alias(&mut kvs_conn.get_connection().unwrap(), gid, key) {
        Ok(_) => {
            if let Err(why) = msg
                .react(
                    &ctx.http,
                    ReactionType::Unicode(REACTION_SUCESSED.to_string()),
                )
                .await
            {
                println!("Error reacting message: {:?}", why);
            };
        }
        Err(why) => println!("Error adding alias: {:?}", why),
    }
    Ok(())
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    let data = _ctx.data.read().await;
    let db = data.get::<DbConn>().unwrap().clone();
    let kvs_conn = data.get::<RedisConn>().unwrap().clone();
    let mut kvs_client = kvs_conn.get_connection().unwrap();

    if let Ok(v) = crud::get_all_commands(
        db.get().unwrap(),
        _msg.guild_id.unwrap().to_string(),
        unknown_command_name.to_string(),
    ) {
        if v.len() != 0 {
            let cmd = v.choose(&mut rand::thread_rng()).unwrap();
            kvs::command_incr(
                &mut kvs_client,
                _msg.guild_id.unwrap().to_string(),
                unknown_command_name.to_string(),
            );
            if let Err(why) = _msg
                .channel_id
                .say(&_ctx.http, cmd.response.to_string())
                .await
            {
                println!("Error sending message: {:?}", why);
            };
        } else {
            match kvs::retrieve_alias(
                &mut kvs_client,
                _msg.guild_id.unwrap().to_string(),
                unknown_command_name.to_string(),
            ) {
                Ok(key) => {
                    if let Ok(v) = crud::get_all_commands(
                        db.get().unwrap(),
                        _msg.guild_id.unwrap().to_string(),
                        key.clone(),
                    ) {
                        if v.len() != 0 {
                            let cmd = v.choose(&mut rand::thread_rng()).unwrap();
                            kvs::command_incr(
                                &mut kvs_client,
                                _msg.guild_id.unwrap().to_string(),
                                key,
                            );
                            if let Err(why) = _msg
                                .channel_id
                                .say(&_ctx.http, cmd.response.to_string())
                                .await
                            {
                                println!("Error sending message: {:?}", why);
                            };
                        }
                    }
                }
                Err(why) => {
                    println!("Error sending message: {:?}", why);
                }
            };
        }
    }
}

#[tokio::main]
async fn main() {
    let conf = match load_config("config.toml".to_string()) {
        Ok(c) => c,
        Err(e) => panic!("fail to perse toml: {}", e),
    };

    let http = Http::new_with_token(&conf.discord_token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .on_mention(Some(bot_id))
                .prefix(&conf.prefix)
                .owners(owners)
        })
        .unrecognised_command(unknown_command)
        .group(&RESP_GROUP);

    let mut client = Client::builder(&conf.discord_token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<DbConn>(
            Pool::builder()
                .build(ConnectionManager::<PgConnection>::new(conf.db_url))
                .unwrap(),
        );
        data.insert::<RedisConn>(open_redis_conn(conf.redis_url).unwrap());
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
