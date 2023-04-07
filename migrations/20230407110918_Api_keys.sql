create table api_keys (
  hash text not null,
  note text,
  created integer not null default (strftime('%s', 'now')),
  primary key (hash)
) strict;
