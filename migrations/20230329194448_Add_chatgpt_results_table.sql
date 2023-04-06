create table statement_predictions (
  statement_id integer not null references statements (id) on delete cascade on update cascade,
  ai_env text not null,
  prompt_name text not null,
  prompt_version integer not null,
  prompt_result text not null,
  completion_tokens integer not null,
  prompt_tokens integer not null,
  total_tokens integer GENERATED ALWAYS AS (completion_tokens + prompt_tokens) VIRTUAL,
  -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  created integer not null default (strftime('%s', 'now')),
  primary key (statement_id, prompt_name, prompt_version)
) strict;
