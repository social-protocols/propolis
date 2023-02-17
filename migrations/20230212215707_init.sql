create table statements (
  id integer not null primary key,
  text text not null
  -- lang text not null
) strict;

create table users (
  id integer not null primary key,
  secret text not null
) strict;

create table authors (
  -- if two statements are the same and were merged, there are multiple authors for one statement
  user_id integer not null references users(id),
  statement_id integer not null references statements(id),
  timestamp integer not null default (strftime('%s', 'now')), -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  FOREIGN KEY (statement_id) REFERENCES statements(id) ON DELETE CASCADE
) strict;

create table votes (
  user_id integer not null references users(id),
  statement_id integer not null references statements(id),
  timestamp integer not null default (strftime('%s', 'now')), -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  vote integer not null check (vote in (-1, 0, 1)), -- or separate table with skipped statements?
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  FOREIGN KEY (statement_id) REFERENCES statements(id) ON DELETE CASCADE
) strict;

create table queue (
  user_id integer not null references users(id),
  statement_id integer not null references statements(id),
  timestamp integer not null default (strftime('%s', 'now')) -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
) strict; 


-- create table translations (
--   original integer not null references statements(id),
--   translation integer not null references statements(id)
-- ) strict;

-- create table specializations (
--   generic integer not null references statements(id),
--   specific integer not null references statements(id)
-- ) strict;

-- create table negations (
--   original integer not null references statements(id),
--   negation integer not null references statements(id)
-- ) strict;

-- /* Pro/Contra arguments for a statement */
-- create table supports (
--   premise integer not null references statements(id),
--   conclusion integer not null references statements(id)
-- ) strict;

-- create view oppositions as
--   select s.premise, n.negation as neg_conclusion
--     from supports s
--     join negations n on s.conclusion = n.original;

