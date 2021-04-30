use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection, QueryResult, RunQueryDsl,
};

use crate::models::{Command, NewCommand};

pub fn add_command(
    conn: PooledConnection<ConnectionManager<PgConnection>>,
    guild_id: String,
    command: String,
    response: String,
    created_by: String,
) -> QueryResult<usize> {
    use crate::schema::commands;

    let new_command = NewCommand {
        guild_id,
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
    gid: String,
    command_key: String,
) -> QueryResult<Vec<Command>> {
    use crate::schema::commands::dsl::*;

    commands
        .filter(guild_id.eq(gid))
        .filter(command.eq(&command_key))
        .load::<Command>(&conn)
}

pub fn command_delete(
    conn: PooledConnection<ConnectionManager<PgConnection>>,
    gid: String,
    command_key: String,
) -> QueryResult<usize> {
    use crate::schema::commands::dsl::*;
    diesel::delete(
        commands
            .filter(guild_id.eq(gid))
            .filter(command.eq(command_key)),
    )
    .execute(&conn)
}
