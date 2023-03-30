PRAGMA recursive_triggers = ON;

CREATE TABLE statements (
  id integer not null primary key, -- rowid
  text text not null,
  created integer not null default (strftime('%s', 'now')) -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
) strict;


CREATE VIRTUAL TABLE statements_fts USING fts5(id UNINDEXED, text);


CREATE TABLE users (
  id integer not null primary key, -- rowid
  secret text not null unique,
  created integer not null default (strftime('%s', 'now')) -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
) strict;

CREATE TABLE authors (
  -- if two statements are the same and were merged, there are multiple authors for one statement
  user_id integer not null references users(id) on delete cascade on update cascade,
  statement_id integer not null references statements(id) on delete cascade on update cascade,
  created integer not null default (strftime('%s', 'now')), -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  primary key (user_id, statement_id)
) strict, without rowid;


CREATE TABLE queue (
  user_id integer not null references users(id) on delete cascade on update cascade,
  statement_id integer not null references statements(id) on delete cascade on update cascade,
  created integer not null default (strftime('%s', 'now')), -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  primary key (user_id, statement_id) on conflict ignore
) strict, without rowid;
CREATE INDEX queue_statement_id on queue (user_id, created, statement_id);


CREATE TABLE IF NOT EXISTS vote_history (
  user_id integer not null references users(id) on delete cascade on update cascade,
  statement_id integer not null references statements(id) on delete cascade on update cascade,
  created integer not null default (strftime('%s', 'now')), -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  vote integer not null -- or separate table with skipped statements?
) strict;
CREATE INDEX vote_history_statement_id on vote_history (user_id, created, statement_id);


CREATE TABLE IF NOT EXISTS votes (
  -- the current vote of a user for a statement, because opinions can change
  statement_id integer not null references statements(id) on delete cascade on update cascade,
  user_id integer not null references users(id) on delete cascade on update cascade,
  vote integer not null, -- or separate table with skipped statements?
  primary key (statement_id, user_id)
) strict, without rowid;


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


CREATE TABLE followups (
  statement_id integer not null references statements(id) on delete cascade on update cascade,
  followup_id integer not null references statements(id) on delete cascade on update cascade,
  target_yes integer not null default 0,
  target_no integer not null default 0,
  primary key (statement_id, followup_id)
) strict, without rowid;


CREATE TABLE subscriptions (
  user_id integer not null references users(id) on delete cascade on update cascade,
  statement_id integer not null references statements(id) on delete cascade on update cascade,
  created integer not null default (strftime('%s', 'now')), -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  primary key (user_id, statement_id) on conflict ignore
) strict, without rowid;

CREATE TRIGGER statements_ai AFTER INSERT ON statements
BEGIN
  -- update search index
  INSERT INTO statements_fts (id, text) VALUES (new.id, new.text);
END;

CREATE TRIGGER statements_ad AFTER DELETE ON statements
BEGIN
  -- update search index
  INSERT INTO statements_fts (statements_fts, id, text) VALUES ('delete', old.id, old.text);
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
