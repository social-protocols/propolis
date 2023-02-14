create table statements (
  id integer not null primary key,
  text text not null
  -- lang text not null
);

create table users (
  id integer not null primary key,
  secret text not null
);

create table votes (
  user_id integer not null references users(id),
  statement_id integer not null references statements(id),
  timestamp timestamp not null default CURRENT_TIMESTAMP,
  vote integer not null check (vote in (-1, 0, 1)), -- or separate table with skipped statements?
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  FOREIGN KEY (statement_id) REFERENCES statements(id) ON DELETE CASCADE
);



-- create table translations (
--   original integer not null references statements(id),
--   translation integer not null references statements(id)
-- );

-- create table specializations (
--   generic integer not null references statements(id),
--   specific integer not null references statements(id)
-- );

-- create table negations (
--   original integer not null references statements(id),
--   negation integer not null references statements(id)
-- );

-- /* Pro/Contra arguments for a statement */
-- create table supports (
--   premise integer not null references statements(id),
--   conclusion integer not null references statements(id)
-- );

-- create view oppositions as
--   select s.premise, n.negation as neg_conclusion
--     from supports s
--     join negations n on s.conclusion = n.original;

