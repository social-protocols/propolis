CREATE TABLE _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    success BOOLEAN NOT NULL,
    checksum BLOB NOT NULL,
    execution_time BIGINT NOT NULL
);
CREATE TABLE statements (
  id integer not null primary key,
  text text not null
  -- lang text not null
) strict;
CREATE TABLE users (
  id integer not null primary key,
  secret text not null unique
) strict;
CREATE TABLE authors (
  -- if two statements are the same and were merged, there are multiple authors for one statement
  user_id integer not null,
  statement_id integer not null,
  timestamp integer not null default (strftime('%s', 'now')), -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  primary key (user_id, statement_id)
) strict;
CREATE VIRTUAL TABLE statements_fts USING fts5(id UNINDEXED, text)
/* statements_fts(id,text) */;
CREATE TABLE IF NOT EXISTS 'statements_fts_data'(id INTEGER PRIMARY KEY, block BLOB);
CREATE TABLE IF NOT EXISTS 'statements_fts_idx'(segid, term, pgno, PRIMARY KEY(segid, term)) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS 'statements_fts_content'(id INTEGER PRIMARY KEY, c0, c1);
CREATE TABLE IF NOT EXISTS 'statements_fts_docsize'(id INTEGER PRIMARY KEY, sz BLOB);
CREATE TABLE IF NOT EXISTS 'statements_fts_config'(k PRIMARY KEY, v) WITHOUT ROWID;
CREATE TRIGGER statements_ai AFTER INSERT ON statements
BEGIN
  INSERT INTO statements_fts (id, text) VALUES (new.id, new.text);
END;
CREATE TABLE followups (
  statement_id integer not null,
  followup_id integer not null,
  primary key (statement_id, followup_id)
) strict, without rowid;
CREATE TABLE queue (
  user_id integer not null,
  statement_id integer not null,
  timestamp integer not null default (strftime('%s', 'now')), -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  primary key (user_id, statement_id)
) strict;
CREATE INDEX queue_statement_id on queue (user_id, timestamp);
CREATE TABLE IF NOT EXISTS "votes" (
  -- the current vote of a user for a statement, because opinions can change
  statement_id integer not null,
  user_id integer not null,
  vote integer not null, -- or separate table with skipped statements?
  primary key (statement_id, user_id)
) strict, without rowid;
CREATE TABLE IF NOT EXISTS "vote_history" (
  user_id integer not null,
  statement_id integer not null,
  timestamp integer not null default (strftime('%s', 'now')), -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  vote integer not null -- or separate table with skipped statements?
) strict;
