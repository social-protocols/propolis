create table api_keys (
  id integer not null primary key,
  hash text not null,
  note text,
  total_tokens integer not null default 0,
  created integer not null default (strftime('%s', 'now'))
) strict;
alter table statement_predictions add column
  api_key_id integer not null references api_keys (id);

CREATE TRIGGER api_key_stats AFTER INSERT ON statement_predictions
  BEGIN
    -- update stats
    UPDATE api_keys
       SET total_tokens = total_tokens + new.total_tokens
     WHERE id = new.api_key_id;
  END;


