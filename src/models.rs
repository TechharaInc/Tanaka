use super::schema::commands;

#[derive(Debug, Queryable)]
pub struct Command {
    pub id: i32,
    pub guild_id: String,
    pub command: String,
    pub response: String,
    pub created_by: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "commands"]
pub struct NewCommand {
    pub guild_id: String,
    pub command: String,
    pub response: String,
    pub created_by: String,
}
