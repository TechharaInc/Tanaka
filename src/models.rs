use chrono::{DateTime, Utc};

use super::schema::commands;

#[derive(Queryable)]
pub struct Commands {
    pub id: i32,
    pub guild_id: String,
    pub command: String,
    pub response: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[table_name = "commands"]
pub struct NewCommand {
    pub guild_id: String,
    pub command: String,
    pub response: String,
    pub created_by: String,
}
