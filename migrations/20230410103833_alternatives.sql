create table alternatives (
  statement_id integer not null references statements (id) on delete cascade on update cascade,
  alternative_id integer not null references statements (id) on delete cascade on update cascade,
  primary key (statement_id, alternative_id)
);
