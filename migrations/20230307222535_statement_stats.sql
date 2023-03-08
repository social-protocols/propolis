create table statement_stats(
  statement_id int not null primary key,
  yes_votes int not null default 0,
  no_votes int not null default 0,
  skip_votes int not null default 0,
  itdepends_votes int not null default 0,
  subscriptions int not null default 0,
  -- computed fields
  total_votes generated always as (yes_votes + no_votes + skip_votes + itdepends_votes) virtual,
  participation generated always as (cast(total_votes - skip_votes as real) / (total_votes)) virtual,
  polarization generated always as (1.0 - cast((abs(yes_votes - no_votes)) as real) / (total_votes - skip_votes)) virtual,
  votes_per_subscription generated always as (cast(total_votes - skip_votes as real) / (subscriptions)) virtual
);

insert into statement_stats (statement_id, yes_votes, no_votes, skip_votes, itdepends_votes)
  select
  statement_id,
  coalesce(sum(vote == 1), 0) as yes_votes,
  coalesce(sum(vote == -1), 0) as no_votes,
  coalesce(sum(vote == 0), 0) as skip_votes,
  coalesce(sum(vote == 2), 0) as itdepends_votes
  from votes group by statement_id
