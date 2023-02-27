create table followups (
  statement_id integer not null,
  followup_id integer not null,
  primary key (statement_id, followup_id)
) strict, without rowid;


-- add missing primary key for queue

-- create backup of original queue
create table queue_backup as select * from queue;
drop table queue;


-- recreate table with correct primary key
create table queue (
  user_id integer not null,
  statement_id integer not null,
  timestamp integer not null default (strftime('%s', 'now')), -- https://stackoverflow.com/questions/11556546/sqlite-storing-default-timestamp-as-unixepoch
  primary key (user_id, statement_id)
) strict; 
create index queue_statement_id on queue (user_id, timestamp);
insert into queue (user_id, statement_id, timestamp) select user_id, statement_id, timestamp from queue_backup where true on conflict do nothing;


drop table queue_backup;
