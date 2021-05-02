use redis::{self, Commands, RedisError};

pub fn command_incr(conn: &mut redis::Connection, gid: String, command_key: String) {
    redis::cmd("ZINCRBY")
        .arg(gid)
        .arg(1)
        .arg(command_key)
        .execute(conn);
}

pub fn command_delete(conn: &mut redis::Connection, gid: String, command_key: String) {
    redis::cmd("ZREM").arg(gid).arg(command_key).execute(conn);
}

pub fn command_rank(conn: &mut redis::Connection, gid: String) -> Vec<(String, u32)> {
    conn.zrevrange_withscores(gid, 0, 10).unwrap()
}

pub fn add_alias(
    conn: &mut redis::Connection,
    gid: String,
    src_command_key: String,
    dst_command_key: String,
) -> std::result::Result<u64, RedisError> {
    conn.hset(
        gid,
        format!("{}:{}", "alias", &dst_command_key),
        src_command_key,
    )
}

pub fn retrieve_alias(
    conn: &mut redis::Connection,
    gid: String,
    dst_command_key: String,
) -> std::result::Result<String, RedisError> {
    conn.hget(gid, format!("{}:{}", "alias", dst_command_key))
}

pub fn remove_alias(
    conn: &mut redis::Connection,
    gid: String,
    dst_command_key: String,
) -> std::result::Result<u64, RedisError> {
    conn.hdel(gid, format!("{}:{}", "alias", dst_command_key))
}
