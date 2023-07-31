DROP TRIGGER votes_ad;
DROP TRIGGER votes_ai;
DROP TRIGGER votes_au;

DROP TRIGGER subscriptions_ai;
CREATE TRIGGER subscriptions_ai AFTER INSERT ON subscriptions
BEGIN
  -- add follow-ups to queue
  INSERT INTO queue (user_id, statement_id)
    SELECT new.user_id, followup_id
    FROM followups
    WHERE statement_id = new.statement_id;
END;

DROP TRIGGER subscriptions_ad;

DROP TABLE statement_stats;

CREATE INDEX subscriptions_idx_statement_id ON subscriptions(statement_id);

CREATE VIEW statement_stats AS
WITH counted_votes as (
    SELECT
        id as statement_id
        , coalesce(sum(vote = 1), 0) as yes_votes
        , coalesce(sum(vote = -1), 0) as no_votes
        , coalesce(sum(vote = 0), 0) as skip_votes
        , coalesce(sum(vote = 2), 0) as itdepends_votes
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
        , (yes_votes + no_votes + skip_votes + itdepends_votes) as total_votes
    from counted_votes_subscriptions
)
select
    *
    , coalesce((cast(total_votes - skip_votes as real) / (total_votes)), 0) as participation
    , coalesce((1.0 - cast((abs(yes_votes - no_votes)) as real) / (total_votes - skip_votes)), 0) as polarization
    , coalesce((cast(total_votes - skip_votes as real) / (subscriptions)), 0) as votes_per_subscription
from cte
;

