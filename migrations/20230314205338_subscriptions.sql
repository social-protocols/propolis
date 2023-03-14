CREATE TABLE subscriptions (
  -- if two statements are the same and were merged, there are multiple authors for one statement
  user_id integer not null,
  statement_id integer not null,
  primary key (user_id, statement_id)
) strict;

INSERT INTO subscriptions (user_id, statement_id) SELECT user_id, statement_id FROM authors;
