CREATE INDEX queue_statement_id on queue (user_id, created, statement_id);
CREATE INDEX subscriptions_idx_statement_id ON subscriptions(statement_id);
CREATE INDEX vote_history_statement_id on vote_history (user_id, created, statement_id);
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
CREATE TABLE relation_types (
    id integer primary key,
    name text not null
);
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
CREATE TABLE statement_relations (
    statement_id integer not null references statements(id),
    related_statement_id integer not null references statements(id),
    relation_type integer not null references relation_types(id),
    created_at timestamp not null default current_timestamp,
    primary key (statement_id, related_statement_id, relation_type)
);
CREATE TABLE statements (
  id integer not null primary key, -- rowid
  text text not null,
  created integer not null default (strftime('%s', 'now')) -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
) strict;
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
CREATE TRIGGER api_key_stats AFTER INSERT ON statement_predictions
  BEGIN
    -- update stats
    UPDATE api_keys
       SET total_tokens = total_tokens + new.total_tokens
     WHERE id = new.api_key_id;
  END;
CREATE TRIGGER followups_ai AFTER INSERT ON followups
BEGIN
  -- for yes/no voters
  INSERT INTO queue (user_id, statement_id)
    SELECT user_id, new.followup_id
    FROM votes
    WHERE statement_id = new.statement_id
    AND (
          ( vote =  1 AND new.target_yes = 1)
       OR ( vote = -1 AND new.target_no  = 1)
    );
  -- for it-depends voters
  INSERT INTO queue (user_id, statement_id)
    SELECT user_id, new.followup_id
    FROM votes
    WHERE statement_id = new.statement_id
    AND vote = 2;
  -- for subscribers
  INSERT INTO queue (user_id, statement_id)
    SELECT user_id, new.followup_id
    FROM subscriptions
    WHERE statement_id = new.statement_id;
END;
CREATE TRIGGER followups_au AFTER UPDATE ON followups
BEGIN
  -- for yes/no voters
  INSERT INTO queue (user_id, statement_id)
    SELECT user_id, new.followup_id
    FROM votes
    WHERE statement_id = new.statement_id
    AND (
          ( vote =  1 AND new.target_yes = 1)
       OR ( vote = -1 AND new.target_no  = 1)
    );
END;
CREATE TRIGGER statement_embeddings_stats AFTER INSERT ON statement_embeddings
  BEGIN
    -- update stats
    UPDATE api_keys
       SET total_tokens = total_tokens + new.total_tokens
     WHERE id = new.api_key_id;
  END;
CREATE TRIGGER statements_ad AFTER DELETE ON statements BEGIN
  -- update search index
  DELETE FROM statements_fts WHERE id = old.id;
END;
CREATE TRIGGER statements_ai AFTER INSERT ON statements
BEGIN
  -- update search index
  INSERT INTO statements_fts (id, text) VALUES (new.id, new.text);
END;
CREATE TRIGGER statements_au AFTER UPDATE ON statements BEGIN
  -- update search index
  DELETE FROM statements_fts WHERE id = old.id;
  insert into statements_fts(id, text) values (new.id, new.text);
END;
CREATE TRIGGER subscriptions_ai AFTER INSERT ON subscriptions
BEGIN
  -- add follow-ups to queue
  INSERT INTO queue (user_id, statement_id)
    SELECT new.user_id, followup_id
    FROM followups
    WHERE statement_id = new.statement_id;
END;
CREATE TRIGGER vote_history_ai AFTER INSERT ON vote_history
BEGIN
    -- update stats
    INSERT INTO votes (statement_id, user_id, vote)
      VALUES (new.statement_id, new.user_id, new.vote)
      on conflict (statement_id, user_id) do update set vote = excluded.vote;
    -- remove from queue
    DELETE FROM queue WHERE user_id = new.user_id AND statement_id = new.statement_id;
END;
CREATE TRIGGER vote_history_ai_itdepends AFTER INSERT ON vote_history
  WHEN new.vote = 2 -- it depends
BEGIN
  -- if the vote was "it depends", add all follow-ups to queue
  INSERT INTO queue (user_id, statement_id)
    SELECT new.user_id, followup_id
    FROM followups
    WHERE statement_id = new.statement_id;
END;
CREATE TRIGGER vote_history_ai_yes AFTER INSERT ON vote_history
  WHEN new.vote = 1 OR new.vote = -1 -- yes or no
BEGIN
  -- if the vote was yes or no, add respective follow-ups to queue
  INSERT INTO queue (user_id, statement_id)
    SELECT new.user_id, followup_id
      FROM followups
      WHERE statement_id = new.statement_id
      AND (
         (new.vote =  1 AND target_yes = 1)
        OR (new.vote = -1 AND target_no  = 1)
      );
END;
CREATE VIEW statement_stats AS
WITH counted_votes as (
    SELECT
        id as statement_id
        , coalesce(sum(vote = 1), 0) as yes_votes
        , coalesce(sum(vote = -1), 0) as no_votes
        , coalesce(sum(vote = 0), 0) as skip_votes
    from statements
    left outer join votes
    on statements.id = votes.statement_id
    group by statement_id
)
, counted_votes_subscriptions as (
    select
        counted_votes.*
        , count(all user_id) as subscriptions
    from counted_votes
    left outer join subscriptions
    using(statement_id)
    group by statement_id
)
, cte as (
    select
        *
        , (yes_votes + no_votes) as total_votes
    from counted_votes_subscriptions
)
select
    *
    , coalesce((cast(total_votes as real) / (total_votes + skip_votes)), 0) as participation
    -- polarization: 1 = 50% yes and 50% no, 0 = 100% yes or 100% no
    , coalesce((1.0 - cast((abs(yes_votes - no_votes)) as real) / (total_votes)), 0) as polarization
    , coalesce((cast(total_votes as real) / (subscriptions)), 0) as votes_per_subscription
from cte
/* statement_stats(statement_id,yes_votes,no_votes,skip_votes,subscriptions,total_votes,participation,polarization,votes_per_subscription) */;
CREATE VIEW vote_stats as select statement_id, vote, count(*) as vote_count from votes group by statement_id, vote
/* vote_stats(statement_id,vote,vote_count) */;
CREATE VIRTUAL TABLE statements_fts USING fts5(id UNINDEXED, text)
/* statements_fts(id,text) */;
