table! {
    commands (id) {
        id -> Int4,
        guild_id -> Text,
        command -> Text,
        response -> Text,
        created_at -> Timestamp,
    }
}
