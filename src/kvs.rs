use redis::{self, Commands, RedisError};
use serenity::model::id::GuildId;

pub fn command_incr(conn: &mut redis::Connection, gid: &GuildId, command_key: &str) {
    redis::cmd("ZINCRBY")
        .arg(format!("{}:{}", gid, "rank"))
        .arg(1)
        .arg(command_key)
        .execute(conn);
}

pub fn command_delete(conn: &mut redis::Connection, gid: &GuildId, command_key: &str) {
    redis::cmd("ZREM")
        .arg(format!("{}:{}", gid, "rank"))
        .arg(command_key)
        .execute(conn);
}

pub fn command_rank(conn: &mut redis::Connection, gid: &GuildId) -> Vec<(String, u32)> {
    conn.zrevrange_withscores(format!("{}:{}", gid, "rank"), 0, 9)
        .unwrap()
}

pub fn add_alias(
    conn: &mut redis::Connection,
    gid: &GuildId,
    src_command_key: &str,
    dst_command_key: &str,
) -> Result<u64, RedisError> {
    conn.hset(
        format!("{}:{}", gid, "alias"),
        src_command_key,
        dst_command_key,
    )
}

pub fn retrieve_alias(
    conn: &mut redis::Connection,
    gid: &GuildId,
    src_command_key: &str,
) -> Result<String, RedisError> {
    conn.hget(format!("{}:{}", gid, "alias"), src_command_key)
}

pub fn remove_alias(
    conn: &mut redis::Connection,
    gid: &GuildId,
    src_command_key: &str,
) -> Result<u64, RedisError> {
    conn.hdel(format!("{}:{}", gid, "alias"), src_command_key)
}
