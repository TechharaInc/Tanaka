use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection, QueryResult, RunQueryDsl,
};
use serenity::model::id::GuildId;

use crate::models::{Command, NewCommand};

pub fn add_command(
    conn: PooledConnection<ConnectionManager<PgConnection>>,
    guild_id: &GuildId,
    command: String,
    response: String,
    created_by: String,
) -> QueryResult<usize> {
    use crate::schema::commands;

    let new_command = NewCommand {
        guild_id: guild_id.to_string(),
        command,
        response,
        created_by,
    };

    diesel::insert_into(commands::table)
        .values(new_command)
        .execute(&conn)
}

pub fn get_all_commands(
    conn: PooledConnection<ConnectionManager<PgConnection>>,
    gid: &GuildId,
    command_key: &str,
) -> QueryResult<Vec<Command>> {
    use crate::schema::commands::dsl::*;

    commands
        .filter(guild_id.eq(gid.to_string()))
        .filter(command.eq(&command_key))
        .load::<Command>(&conn)
}

pub fn command_delete(
    conn: PooledConnection<ConnectionManager<PgConnection>>,
    gid: &GuildId,
    command_key: &String,
) -> QueryResult<usize> {
    use crate::schema::commands::dsl::*;
    let gid_str = gid.to_string();
    diesel::delete(
        commands
            .filter(guild_id.eq(gid_str))
            .filter(command.eq(command_key)),
    )
    .execute(&conn)
}
