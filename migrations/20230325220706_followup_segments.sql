create table followups_tmp as select * from followups;
drop table followups;

create table followups (
  statement_id integer not null,
  followup_id integer not null,
  target_yes integer not null default 0,
  target_no integer not null default 0,
  primary key (statement_id, followup_id)
) strict, without rowid;

insert into followups select statement_id, followup_id, 1, 1 from followups_tmp;
drop table followups_tmp;


