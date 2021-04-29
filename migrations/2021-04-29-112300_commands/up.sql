-- Your SQL goes here
CREATE TABLE commands (
  id SERIAL,
  guild_id TEXT NOT NULL,
  command TEXT NOT NULL,
  response TEXT NOT NULL,
  created_by TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id)
);
