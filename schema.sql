CREATE INDEX queue_statement_id on queue (user_id, created, statement_id);
CREATE INDEX vote_history_statement_id on vote_history (user_id, created, statement_id);
CREATE TABLE alternatives (
  statement_id integer not null references statements (id) on delete cascade on update cascade,
  alternative_id integer not null references statements (id) on delete cascade on update cascade,
  primary key (statement_id, alternative_id)
);
CREATE TABLE api_keys (
  id integer not null primary key,
  hash text not null,
  note text,
  total_tokens integer not null default 0,
  created integer not null default (strftime('%s', 'now'))
) strict;
CREATE TABLE authors (
  -- if two statements are the same and were merged, there are multiple authors for one statement
  user_id integer not null references users(id) on delete cascade on update cascade,
  statement_id integer not null references statements(id) on delete cascade on update cascade,
  created integer not null default (strftime('%s', 'now')), -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  primary key (user_id, statement_id)
) strict, without rowid;
CREATE TABLE followups (
  statement_id integer not null references statements(id) on delete cascade on update cascade,
  followup_id integer not null references statements(id) on delete cascade on update cascade,
  target_yes integer not null default 0,
  target_no integer not null default 0,
  primary key (statement_id, followup_id)
) strict, without rowid;
CREATE TABLE IF NOT EXISTS 'statements_fts_config'(k PRIMARY KEY, v) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS 'statements_fts_content'(id INTEGER PRIMARY KEY, c0, c1);
CREATE TABLE IF NOT EXISTS 'statements_fts_data'(id INTEGER PRIMARY KEY, block BLOB);
CREATE TABLE IF NOT EXISTS 'statements_fts_docsize'(id INTEGER PRIMARY KEY, sz BLOB);
CREATE TABLE IF NOT EXISTS 'statements_fts_idx'(segid, term, pgno, PRIMARY KEY(segid, term)) WITHOUT ROWID;
CREATE TABLE queue (
  user_id integer not null references users(id) on delete cascade on update cascade,
  statement_id integer not null references statements(id) on delete cascade on update cascade,
  created integer not null default (strftime('%s', 'now')), -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  primary key (user_id, statement_id) on conflict ignore
) strict, without rowid;
CREATE TABLE _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    success BOOLEAN NOT NULL,
    checksum BLOB NOT NULL,
    execution_time BIGINT NOT NULL
);
CREATE TABLE statement_embeddings (
  statement_id integer not null references statements(id) on delete cascade on update cascade,
  data blob,
  prompt_tokens integer not null,
  total_tokens integer GENERATED ALWAYS AS (prompt_tokens) VIRTUAL,
  api_key_id integer not null references api_keys (id),
  created integer not null default (strftime('%s', 'now')),
  primary key (statement_id)
) strict;
CREATE TABLE statement_flags (
  statement_id integer not null references statements(id) on delete cascade on update cascade,
  -- fully flagged or maybe flagged
  state integer not null,
  -- contains serialized json
  categories text not null,
  -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  created integer not null default (strftime('%s', 'now')),
  primary key (statement_id)
) strict;
CREATE TABLE statement_predictions (
  statement_id integer not null references statements (id) on delete cascade on update cascade,
  ai_env text not null,
  prompt_name text not null,
  prompt_version integer not null,
  prompt_result text not null,
  completion_tokens integer not null,
  prompt_tokens integer not null,
  total_tokens integer GENERATED ALWAYS AS (completion_tokens + prompt_tokens) VIRTUAL,
  -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  created integer not null default (strftime('%s', 'now')), api_key_id integer not null references api_keys (id),
  primary key (statement_id, prompt_name, prompt_version)
) strict;
CREATE TABLE statements (
  id integer not null primary key, -- rowid
  text text not null,
  created integer not null default (strftime('%s', 'now')) -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
) strict;
CREATE TABLE statement_stats(
  statement_id int not null primary key references statements(id) on delete cascade on update cascade,
  yes_votes int not null default 0,
  no_votes int not null default 0,
  skip_votes int not null default 0,
  itdepends_votes int not null default 0,
  subscriptions int not null default 0,
  -- computed fields
  total_votes generated always as (yes_votes + no_votes + skip_votes + itdepends_votes) virtual,
  participation generated always as (cast(total_votes - skip_votes as real) / (total_votes)) virtual,
  polarization generated always as (1.0 - cast((abs(yes_votes - no_votes)) as real) / (total_votes - skip_votes)) virtual,
  votes_per_subscription generated always as (cast(total_votes - skip_votes as real) / (subscriptions)) virtual
);
CREATE TABLE subscriptions (
  user_id integer not null references users(id) on delete cascade on update cascade,
  statement_id integer not null references statements(id) on delete cascade on update cascade,
  created integer not null default (strftime('%s', 'now')), -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  primary key (user_id, statement_id) on conflict ignore
) strict, without rowid;
CREATE TABLE users (
  id integer not null primary key, -- rowid
  secret text not null unique,
  created integer not null default (strftime('%s', 'now')) -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
) strict;
CREATE TABLE vote_history (
  user_id integer not null references users(id) on delete cascade on update cascade,
  statement_id integer not null references statements(id) on delete cascade on update cascade,
  created integer not null default (strftime('%s', 'now')), -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  vote integer not null -- or separate table with skipped statements?
) strict;
CREATE TABLE votes (
  -- the current vote of a user for a statement, because opinions can change
  statement_id integer not null references statements(id) on delete cascade on update cascade,
  user_id integer not null references users(id) on delete cascade on update cascade,
  vote integer not null, -- or separate table with skipped statements?
  primary key (statement_id, user_id)
) strict, without rowid;
CREATE VIRTUAL TABLE statements_fts USING fts5(id UNINDEXED, text)
/* statements_fts(id,text) */;
