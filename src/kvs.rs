use redis::{self, Commands};

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
    conn.zrangebyscore_withscores(gid, 0, 10).unwrap()
}
