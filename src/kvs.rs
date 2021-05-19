use redis::{self, Commands, RedisError};

pub fn command_incr(conn: &mut redis::Connection, gid: String, command_key: String) {
    redis::cmd("ZINCRBY")
        .arg(format!("{}:{}", gid, "rank"))
        .arg(1)
        .arg(command_key)
        .execute(conn);
}

pub fn command_delete(conn: &mut redis::Connection, gid: String, command_key: String) {
    redis::cmd("ZREM")
        .arg(format!("{}:{}", gid, "rank"))
        .arg(command_key)
        .execute(conn);
}

pub fn command_rank(conn: &mut redis::Connection, gid: String) -> Vec<(String, u32)> {
    conn.zrevrange_withscores(format!("{}:{}", gid, "rank"), 0, 9)
        .unwrap()
}

pub fn add_alias(
    conn: &mut redis::Connection,
    gid: String,
    src_command_key: String,
    dst_command_key: String,
) -> Result<u64, RedisError> {
    conn.hset(
        format!("{}:{}", gid, "alias"),
        src_command_key,
        dst_command_key,
    )
}

pub fn retrieve_alias(
    conn: &mut redis::Connection,
    gid: String,
    src_command_key: String,
) -> Result<String, RedisError> {
    conn.hget(format!("{}:{}", gid, "alias"), src_command_key)
}

pub fn remove_alias(
    conn: &mut redis::Connection,
    gid: String,
    src_command_key: String,
) -> Result<u64, RedisError> {
    conn.hdel(format!("{}:{}", gid, "alias"), src_command_key)
}
