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
    OptionalExtension, RunQueryDsl,
};

struct DbConn;

impl TypeMapKey for DbConn {
    type Value = Pool<ConnectionManager<PgConnection>>;
}

use crate::kvs::Database;
mod kvs;

mod consts;
use self::models::NewCommand;
use crate::consts::consts::{REACTION_FAILED, REACTION_SUCESSED};

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
    use schema::commands;

    if args.len() < 2 && msg.attachments.is_empty() {
        if let Err(why) = msg.channel_id.say(&ctx.http, "引数が足りません").await {
            println!("Error sending message: {:?}", why);
        }
    } else {
        let db = &establish_connection();
        // let data = ctx.data.read().await;
        // let db = data.get::<DbConn>().unwrap();

        let key = args.single::<String>().unwrap();
        let value = if msg.attachments.is_empty() {
            args.rest()
        } else {
            &msg.attachments[0].url
        };

        let new_command = NewCommand {
            guild_id: format!("{:?}", msg.guild_id),
            command: key,
            response: value.to_string(),
            created_by: format!("{}", msg.author.id),
        };

        let emoji = match diesel::insert_into(commands::table)
            .values(&new_command)
            .get_result(db)
        {
            Ok(_) => REACTION_SUCESSED,
            Err(_) => REACTION_FAILED,
        };

        // let emoji = match db.put(
        //     format!("{:?}:{}", msg.guild_id, key).as_bytes(),
        //     value.as_bytes(),
        // ) {
        //     Ok(()) => REACTION_SUCESSED,
        //     Err(_) => REACTION_FAILED,
        // };
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
        let db = data.get::<Database>().unwrap();

        let emoji = match db
            .delete(format!("{:?}:{}", msg.guild_id, args.single::<String>().unwrap()).as_bytes())
        {
            Ok(()) => REACTION_SUCESSED,
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
    let db = data.get::<Database>().unwrap();
    if let Ok(Some(v)) = db.get(format!("{:?}:{}", _msg.guild_id, unknown_command_name).as_bytes())
    {
        if v.len() != 0 {
            if let Err(why) = _msg
                .channel_id
                .say(&_ctx.http, String::from_utf8(v).unwrap())
                .await
            {
                if let Err(why) = _msg
                    .react(
                        &_ctx.http,
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

    // {
    //     let data = client.data.write();
    //     data.await.insert::<DbConn>(
    //         Pool::builder()
    //             .build(ConnectionManager::<PgConnection>::new(conf.db_url))
    //             .unwrap(),
    //     );
    // }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
