-- Your SQL goes here
CREATE TABLE commands (
  id SERIAL,
  guild_id TEXT NOT NULL,
  command TEXT NOT NULL,
  response TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  PRIMARY KEY (id)
);
