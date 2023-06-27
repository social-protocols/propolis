-- current schema:
-- CREATE TABLE statement_stats(
--   statement_id int not null primary key references statements(id) on delete cascade on update cascade,
--   yes_votes int not null default 0,
--   no_votes int not null default 0,
--   skip_votes int not null default 0,
--   itdepends_votes int not null default 0,
--   subscriptions int not null default 0,
--   -- computed fields
--   total_votes generated always as (yes_votes + no_votes + skip_votes + itdepends_votes) virtual,
--   participation generated always as (cast(total_votes - skip_votes as real) / (total_votes)) virtual,
--   polarization generated always as (1.0 - cast((abs(yes_votes - no_votes)) as real) / (total_votes - skip_votes)) virtual,
--   votes_per_subscription generated always as (cast(total_votes - skip_votes as real) / (subscriptions)) virtual
-- );

-- add new column unclear_votes:
ALTER TABLE statement_stats ADD COLUMN unclear_votes int not null default 0;


-- replace total_votes to include new column unclear_votes:
-- remove dependent columns first, then recreate them:

ALTER TABLE statement_stats DROP COLUMN participation;
ALTER TABLE statement_stats DROP COLUMN polarization;
ALTER TABLE statement_stats DROP COLUMN votes_per_subscription;
ALTER TABLE statement_stats DROP COLUMN total_votes;
ALTER TABLE statement_stats ADD COLUMN total_votes generated always as (yes_votes + no_votes + skip_votes + itdepends_votes + unclear_votes) virtual;
ALTER TABLE statement_stats ADD COLUMN participation generated always as (cast(total_votes - skip_votes as real) / (total_votes)) virtual;
ALTER TABLE statement_stats ADD COLUMN polarization generated always as (1.0 - cast((abs(yes_votes - no_votes)) as real) / (total_votes - skip_votes)) virtual;
ALTER TABLE statement_stats ADD COLUMN votes_per_subscription generated always as (cast(total_votes - skip_votes as real) / (subscriptions)) virtual;


/* current triggers:

CREATE TRIGGER statements_ai AFTER INSERT ON statements
BEGIN
  -- update search index
  INSERT INTO statements_fts (id, text) VALUES (new.id, new.text);
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
CREATE TRIGGER vote_history_ai_itdepends AFTER INSERT ON vote_history
  WHEN new.vote = 2 -- it depends
BEGIN
  -- if the vote was "it depends", add all follow-ups to queue
  INSERT INTO queue (user_id, statement_id)
    SELECT new.user_id, followup_id
    FROM followups
    WHERE statement_id = new.statement_id;
END;
CREATE TRIGGER votes_ai AFTER INSERT ON votes
BEGIN
  -- update stats
  INSERT INTO statement_stats (statement_id, yes_votes, no_votes, skip_votes, itdepends_votes)
  VALUES (
    new.statement_id,
    (new.vote = 1),
    (new.vote = -1),
    (new.vote = 0),
    (new.vote = 2)
  )
  on CONFLICT (statement_id)
  do UPDATE SET
    yes_votes = statement_stats.yes_votes + (new.vote = 1),
    no_votes = statement_stats.no_votes + (new.vote = -1),
    skip_votes = statement_stats.skip_votes + (new.vote = 0),
    itdepends_votes = statement_stats.itdepends_votes + (new.vote = 2);
END;
CREATE TRIGGER votes_au AFTER UPDATE ON votes
BEGIN
  -- update stats
  UPDATE statement_stats
   SET yes_votes = statement_stats.yes_votes + (new.vote = 1) - (old.vote = 1),
     no_votes = statement_stats.no_votes + (new.vote = -1) - (old.vote = -1),
     skip_votes = statement_stats.skip_votes + (new.vote = 0) - (old.vote = 0),
     itdepends_votes = statement_stats.itdepends_votes + (new.vote = 2) - (old.vote = 2)
   WHERE statement_id = old.statement_id;
END;
CREATE TRIGGER votes_ad AFTER DELETE ON votes
BEGIN
  -- update stats
  UPDATE statement_stats
   SET yes_votes = statement_stats.yes_votes - (old.vote = 1),
     no_votes = statement_stats.no_votes - (old.vote = -1),
     skip_votes = statement_stats.skip_votes - (old.vote = 0),
     itdepends_votes = statement_stats.itdepends_votes - (old.vote = 2)
   WHERE statement_id = old.statement_id;
END;
CREATE TRIGGER subscriptions_ai AFTER INSERT ON subscriptions
BEGIN
  --  update stats
  INSERT INTO statement_stats (statement_id, subscriptions)
    VALUES (new.statement_id, 1)
    on CONFLICT (statement_id)
    do UPDATE SET subscriptions = statement_stats.subscriptions + 1;
  -- add follow-ups to queue
  INSERT INTO queue (user_id, statement_id)
    SELECT new.user_id, followup_id
    FROM followups
    WHERE statement_id = new.statement_id;
END;
CREATE TRIGGER subscriptions_ad AFTER DELETE ON subscriptions
BEGIN
  -- update stats
  UPDATE statement_stats
   SET subscriptions = statement_stats.subscriptions - 1
   WHERE statement_id = old.statement_id;
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
CREATE TABLE api_keys (
  id integer not null primary key,
  hash text not null,
  note text,
  total_tokens integer not null default 0,
  created integer not null default (strftime('%s', 'now'))
) strict;
CREATE TRIGGER api_key_stats AFTER INSERT ON statement_predictions
  BEGIN
    -- update stats
    UPDATE api_keys
       SET total_tokens = total_tokens + new.total_tokens
     WHERE id = new.api_key_id;
  END;
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
CREATE TABLE alternatives (
  statement_id integer not null references statements (id) on delete cascade on update cascade,
  alternative_id integer not null references statements (id) on delete cascade on update cascade,
  primary key (statement_id, alternative_id)
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
CREATE TRIGGER statements_au AFTER UPDATE ON statements BEGIN
  -- update search index
  DELETE FROM statements_fts WHERE id = old.id;
  insert into statements_fts(id, text) values (new.id, new.text);
END;
*/

-- update respective triggers to include unclear votes:
DROP TRIGGER votes_ai;
CREATE TRIGGER votes_ai AFTER INSERT ON votes
BEGIN
  -- update stats
  INSERT INTO statement_stats (statement_id, yes_votes, no_votes, skip_votes, itdepends_votes, unclear_votes)
    VALUES (new.statement_id, (new.vote = 1), (new.vote = -1), (new.vote = 0), (new.vote = 2), (new.vote = 3))
    on CONFLICT (statement_id)
    do UPDATE SET
      yes_votes = statement_stats.yes_votes + (new.vote = 1),
      no_votes = statement_stats.no_votes + (new.vote = -1),
      skip_votes = statement_stats.skip_votes + (new.vote = 0),
      itdepends_votes = statement_stats.itdepends_votes + (new.vote = 2),
      unclear_votes = statement_stats.unclear_votes + (new.vote = 3);
END;
DROP TRIGGER votes_au;
CREATE TRIGGER votes_au AFTER UPDATE ON votes
BEGIN
  -- update stats
  UPDATE statement_stats
   SET
     yes_votes = statement_stats.yes_votes + (new.vote = 1) - (old.vote = 1),
     no_votes = statement_stats.no_votes + (new.vote = -1) - (old.vote = -1),
     skip_votes = statement_stats.skip_votes + (new.vote = 0) - (old.vote = 0),
     itdepends_votes = statement_stats.itdepends_votes + (new.vote = 2) - (old.vote = 2),
     unclear_votes = statement_stats.unclear_votes + (new.vote = 3) - (old.vote = 3)
   WHERE statement_id = old.statement_id;
END;
DROP TRIGGER votes_ad;
CREATE TRIGGER votes_ad AFTER DELETE ON votes
BEGIN
  -- update stats
  UPDATE statement_stats
   SET
     yes_votes = statement_stats.yes_votes - (old.vote = 1),
     no_votes = statement_stats.no_votes - (old.vote = -1),
     skip_votes = statement_stats.skip_votes - (old.vote = 0),
     itdepends_votes = statement_stats.itdepends_votes - (old.vote = 2),
     unclear_votes = statement_stats.unclear_votes - (old.vote = 3)
   WHERE statement_id = old.statement_id;
END;
