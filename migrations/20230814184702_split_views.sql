create view vote_stats as select statement_id, vote, count(*) as vote_count from votes group by statement_id, vote;


drop view statement_stats;
CREATE VIEW statement_stats AS
WITH counted_votes as (
    SELECT
        id as statement_id
        , coalesce(sum(vote = 1), 0) as yes_votes
        , coalesce(sum(vote = -1), 0) as no_votes
        , coalesce(sum(vote = 0), 0) as skip_votes
    from statements
    left outer join votes
    on statements.id = votes.statement_id
    group by statement_id
)
, counted_votes_subscriptions as (
    select
        counted_votes.*
        , count(all user_id) as subscriptions
    from counted_votes
    left outer join subscriptions
    using(statement_id)
    group by statement_id
)
, cte as (
    select
        *
        , (yes_votes + no_votes) as total_votes
    from counted_votes_subscriptions
)
select
    *
    , coalesce((cast(total_votes as real) / (total_votes + skip_votes)), 0) as participation
    -- polarization: 1 = 50% yes and 50% no, 0 = 100% yes or 100% no
    , coalesce((1.0 - cast((abs(yes_votes - no_votes)) as real) / (total_votes)), 0) as polarization
    , coalesce((cast(total_votes as real) / (subscriptions)), 0) as votes_per_subscription
from cte;

drop table alternatives;

-- sqlite

create table relation_types (
    id integer primary key,
    name text not null
);

create table statement_relations (
    statement_id integer not null references statements(id),
    related_statement_id integer not null references statements(id),
    relation_type integer not null references relation_types(id),
    created_at timestamp not null default current_timestamp,
    primary key (statement_id, related_statement_id, relation_type)
);
