create table statements (
  id integer not null primary key,
  text text not null
  -- lang text not null
) strict;

create table users (
  id integer not null primary key,
  secret text not null unique
) strict;

create table authors (
  -- if two statements are the same and were merged, there are multiple authors for one statement
  user_id integer not null,
  statement_id integer not null,
  timestamp integer not null default (strftime('%s', 'now')), -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  primary key (user_id, statement_id)
) strict;


create table votes (
  -- the current vote of a user for a statement, because opinions can change
  statement_id integer not null,
  user_id integer not null,
  vote integer not null check (vote in (-1, 0, 1)), -- or separate table with skipped statements?
  primary key (statement_id, user_id)
) strict, without rowid;

create table vote_history (
  user_id integer not null,
  statement_id integer not null,
  timestamp integer not null default (strftime('%s', 'now')), -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  vote integer not null check (vote in (-1, 0, 1)) -- or separate table with skipped statements?
) strict;

create table queue (
  user_id integer not null,
  statement_id integer not null,
  timestamp integer not null default (strftime('%s', 'now')) -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
) strict; 


-- create table translations (
--   original integer not null,
--   translation integer not null
-- ) strict;

-- create table specializations (
--   generic integer not null,
--   specific integer not null
-- ) strict;

-- create table negations (
--   original integer not null,
--   negation integer not null
-- ) strict;

-- /* Pro/Contra arguments for a statement */
-- create table supports (
--   premise integer not null,
--   conclusion integer not null
-- ) strict;

-- create view oppositions as
--   select s.premise, n.negation as neg_conclusion
--     from supports s
--     join negations n on s.conclusion = n.original;

