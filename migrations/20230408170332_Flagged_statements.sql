create table statement_flags (
  statement_id integer not null references statements(id) on delete cascade on update cascade,
  -- fully flagged or maybe flagged
  state integer not null,
  -- contains serialized json
  categories text not null,
  -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  created integer not null default (strftime('%s', 'now')),
  primary key (statement_id)
) strict;
