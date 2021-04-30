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
#[commands(add, remove)]
struct Resp;

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
        let conn = db.get().unwrap();

        let emoji = match crud::command_delete(
            conn,
            msg.guild_id.unwrap().to_string(),
            args.single::<String>().unwrap(),
        ) {
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

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    let data = _ctx.data.read().await;
    let db = data.get::<DbConn>().unwrap().clone();
    let conn = db.get().unwrap();

    if let Ok(v) = crud::get_all_commands(
        conn,
        _msg.guild_id.unwrap().to_string(),
        unknown_command_name.to_string(),
    ) {
        if v.len() != 0 {
            let cmd = v.choose(&mut rand::thread_rng()).unwrap();
            if let Err(why) = _msg.channel_id.say(&_ctx.http, cmd.response.clone()).await {
                println!("Error sending message: {:?}", why);
            }
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
        let data = client.data.write();
        data.await.insert::<DbConn>(
            Pool::builder()
                .build(ConnectionManager::<PgConnection>::new(conf.db_url))
                .unwrap(),
        );
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
