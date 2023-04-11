create table statement_embeddings (
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
