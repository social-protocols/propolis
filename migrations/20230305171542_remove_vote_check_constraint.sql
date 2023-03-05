CREATE TABLE votes_new (
  -- the current vote of a user for a statement, because opinions can change
  statement_id integer not null,
  user_id integer not null,
  vote integer not null, -- or separate table with skipped statements?
  primary key (statement_id, user_id)
) strict, without rowid;

CREATE TABLE vote_history_new (
  user_id integer not null,
  statement_id integer not null,
  timestamp integer not null default (strftime('%s', 'now')), -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  vote integer not null -- or separate table with skipped statements?
) strict;

-- copy old tables into new schema
INSERT INTO votes_new (statement_id, user_id, vote)
  SELECT statement_id, user_id, vote FROM votes;
INSERT INTO vote_history_new (user_id, statement_id, vote)
  SELECT user_id, statement_id, vote FROM vote_history;

-- drop old tables
DROP TABLE votes;
DROP TABLE vote_history;

-- rename new tables
ALTER TABLE votes_new RENAME TO votes;
ALTER TABLE vote_history_new RENAME TO vote_history;
