create table statement_predictions (
  statement_id integer not null,
  ai_env text not null,
  prompt_name text not null,
  prompt_version integer not null,
  prompt_result text not null,
  completion_tokens integer not null,
  prompt_tokens integer not null,
  total_tokens integer GENERATED ALWAYS AS (completion_tokens + prompt_tokens) VIRTUAL,
  -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  timestamp integer not null default (strftime('%s', 'now'))
) strict;
