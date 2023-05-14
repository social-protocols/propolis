use sqlx::SqlitePool;

#[sqlx::test]
async fn update_current_vote_and_stats(pool: SqlitePool) -> sqlx::Result<()> {
    sqlx::query!("insert into users(id, secret) values (3, 'abc')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (2, 'The world is flat.')")
        .execute(&pool)
        .await?;

    //////////////////////////////
    // add initial vote
    sqlx::query!("insert into vote_history(user_id, statement_id, vote) values (3, 2, 1)")
        .execute(&pool)
        .await?;

    // expect updated current vote
    let current_vote =
        sqlx::query_scalar!("select vote from votes where user_id = 3 and statement_id = 2")
            .fetch_one(&pool)
            .await?;
    assert_eq!(current_vote, 1);

    // expect updated stats
    let stats =
        sqlx::query!("select yes_votes, no_votes from statement_stats where statement_id = 2")
            .fetch_one(&pool)
            .await?;
    assert_eq!(stats.yes_votes, 1);
    assert_eq!(stats.no_votes, 0);

    //////////////////////////////
    // add changed vote
    sqlx::query!("insert into vote_history(user_id, statement_id, vote) values (3, 2, -1)")
        .execute(&pool)
        .await?;

    // expect updated current vote
    let current_vote =
        sqlx::query_scalar!("select vote from votes where user_id = 3 and statement_id = 2")
            .fetch_one(&pool)
            .await?;
    assert_eq!(current_vote, -1);

    // expect updated stats
    let stats = sqlx::query!(
        "select yes_votes, no_votes, itdepends_votes from statement_stats where statement_id = 2"
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(stats.yes_votes, 0);
    assert_eq!(stats.no_votes, 1);
    assert_eq!(stats.itdepends_votes, 0);

    //////////////////////////////
    // add vote by other user
    sqlx::query!("insert into users(id, secret) values (4, 'efg')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into vote_history(user_id, statement_id, vote) values (4, 2, -1)")
        .execute(&pool)
        .await?;

    // expect updated stats
    let stats = sqlx::query!(
        "select yes_votes, no_votes, itdepends_votes from statement_stats where statement_id = 2"
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(stats.yes_votes, 0);
    assert_eq!(stats.no_votes, 2);
    assert_eq!(stats.itdepends_votes, 0);

    //////////////////////////////
    // add other vote by other user
    sqlx::query!("insert into users(id, secret) values (5, 'xyz')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into vote_history(user_id, statement_id, vote) values (5, 2, 2)")
        .execute(&pool)
        .await?;

    // expect updated stats
    let stats = sqlx::query!(
        "select yes_votes, no_votes, itdepends_votes from statement_stats where statement_id = 2"
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(stats.yes_votes, 0);
    assert_eq!(stats.no_votes, 2);
    assert_eq!(stats.itdepends_votes, 1);

    Ok(())
}

#[sqlx::test]
async fn update_stats_for_subscriptions(pool: SqlitePool) -> sqlx::Result<()> {
    sqlx::query!("insert into users(id, secret) values (3, 'abc')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (2, 'The world is flat.')")
        .execute(&pool)
        .await?;

    //////////////////////////////
    // subscribe
    sqlx::query!("insert into subscriptions(user_id, statement_id) values (3, 2)")
        .execute(&pool)
        .await?;

    // expect updated stats
    let stats = sqlx::query!("select subscriptions from statement_stats where statement_id = 2")
        .fetch_one(&pool)
        .await?;
    assert_eq!(stats.subscriptions, 1);

    //////////////////////////////
    // unsubscribe
    sqlx::query!("delete from subscriptions where user_id = 3 and statement_id = 2")
        .execute(&pool)
        .await?;

    // expect updated stats
    let stats = sqlx::query!("select subscriptions from statement_stats where statement_id = 2")
        .fetch_one(&pool)
        .await?;
    assert_eq!(stats.subscriptions, 0);
    Ok(())
}

#[sqlx::test]
async fn queue_add_existing(pool: SqlitePool) -> sqlx::Result<()> {
    sqlx::query!("insert into users(id, secret) values (3, 'abc')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (2, 'The world is flat.')")
        .execute(&pool)
        .await?;

    sqlx::query!("insert into queue(user_id, statement_id) values (3, 2)")
        .execute(&pool)
        .await?;

    sqlx::query!("insert into queue(user_id, statement_id) values (3, 2)")
        .execute(&pool)
        .await?;

    Ok(())
}

#[sqlx::test]
async fn vote_removes_from_queue(pool: SqlitePool) -> sqlx::Result<()> {
    sqlx::query!("insert into users(id, secret) values (3, 'abc')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (2, 'The world is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into queue(user_id, statement_id) values (3, 2)")
        .execute(&pool)
        .await?;

    //////////////////////////////
    // add initial vote
    sqlx::query!("insert into vote_history(user_id, statement_id, vote) values (3, 2, 1)")
        .execute(&pool)
        .await?;

    // expect statement removed from queue
    let count =
        sqlx::query_scalar!("select count(*) from queue where user_id = 3 and statement_id = 2")
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 0);

    Ok(())
}

#[sqlx::test]
async fn vote_adds_yes_followup_to_queue(pool: SqlitePool) -> sqlx::Result<()> {
    sqlx::query!("insert into users(id, secret) values (17, 'abc')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (2, 'The world is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (3, 'The universe is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into followups(statement_id, followup_id, target_yes, target_no) values (2, 3, 1, 0)")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (4, 'The world is round.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into followups(statement_id, followup_id, target_yes, target_no) values (2, 4, 0, 1)")
        .execute(&pool)
        .await?;

    //////////////////////////////
    // add yes vote
    sqlx::query!("insert into vote_history(user_id, statement_id, vote) values (17, 2, 1)")
        .execute(&pool)
        .await?;

    // expect yes-followup statement in queue
    let count =
        sqlx::query_scalar!("select count(*) from queue where user_id = 17 and statement_id = 3")
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 1);
    let count =
        sqlx::query_scalar!("select count(*) from queue where user_id = 17 and statement_id = 4")
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 0);

    Ok(())
}

#[sqlx::test]
async fn vote_adds_no_followup_to_queue(pool: SqlitePool) -> sqlx::Result<()> {
    sqlx::query!("insert into users(id, secret) values (17, 'abc')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (2, 'The world is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (3, 'The universe is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into followups(statement_id, followup_id, target_yes, target_no) values (2, 3, 1, 0)")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (4, 'The world is round.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into followups(statement_id, followup_id, target_yes, target_no) values (2, 4, 0, 1)")
        .execute(&pool)
        .await?;

    //////////////////////////////
    // add no vote
    sqlx::query!("insert into vote_history(user_id, statement_id, vote) values (17, 2, -1)")
        .execute(&pool)
        .await?;

    // expect no-followup statement in queue
    let count =
        sqlx::query_scalar!("select count(*) from queue where user_id = 17 and statement_id = 3")
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 0);
    let count =
        sqlx::query_scalar!("select count(*) from queue where user_id = 17 and statement_id = 4")
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 1);

    Ok(())
}

#[sqlx::test]
async fn vote_adds_itdepends_followup_to_queue(pool: SqlitePool) -> sqlx::Result<()> {
    sqlx::query!("insert into users(id, secret) values (17, 'abc')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (2, 'The world is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (3, 'The universe is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into followups(statement_id, followup_id, target_yes, target_no) values (2, 3, 1, 0)")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (4, 'The world is round.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into followups(statement_id, followup_id, target_yes, target_no) values (2, 4, 0, 1)")
        .execute(&pool)
        .await?;

    //////////////////////////////
    // add it-depends vote
    sqlx::query!("insert into vote_history(user_id, statement_id, vote) values (17, 2, 2)")
        .execute(&pool)
        .await?;

    // expect both followup statements in queue
    let count =
        sqlx::query_scalar!("select count(*) from queue where user_id = 17 and statement_id = 3")
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 1);
    let count =
        sqlx::query_scalar!("select count(*) from queue where user_id = 17 and statement_id = 4")
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 1);

    Ok(())
}

//adding a followups, puts the followup in the queue for yes voters
#[sqlx::test]
async fn followup_adds_yes_followup_to_queue(pool: SqlitePool) -> sqlx::Result<()> {
    sqlx::query!("insert into users(id, secret) values (17, 'abc')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (2, 'The world is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (3, 'The universe is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (4, 'The world is round.')")
        .execute(&pool)
        .await?;

    //////////////////////////////
    // add yes vote
    sqlx::query!("insert into vote_history(user_id, statement_id, vote) values (17, 2, 1)")
        .execute(&pool)
        .await?;

    // add followup
    sqlx::query!("insert into followups(statement_id, followup_id, target_yes, target_no) values (2, 3, 1, 0)")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into followups(statement_id, followup_id, target_yes, target_no) values (2, 4, 0, 1)")
        .execute(&pool)
        .await?;

    // expect yes-followup statement in queue
    let count =
        sqlx::query_scalar!("select count(*) from queue where user_id = 17 and statement_id = 3")
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 1);
    let count =
        sqlx::query_scalar!("select count(*) from queue where user_id = 17 and statement_id = 4")
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 0);

    Ok(())
}

//adding a followups, puts the followup in the queue for no voters
#[sqlx::test]
async fn followup_adds_no_followup_to_queue(pool: SqlitePool) -> sqlx::Result<()> {
    sqlx::query!("insert into users(id, secret) values (17, 'abc')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (2, 'The world is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (3, 'The universe is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (4, 'The world is round.')")
        .execute(&pool)
        .await?;

    //////////////////////////////
    // add no vote
    sqlx::query!("insert into vote_history(user_id, statement_id, vote) values (17, 2, -1)")
        .execute(&pool)
        .await?;

    // add followup
    sqlx::query!("insert into followups(statement_id, followup_id, target_yes, target_no) values (2, 3, 1, 0)")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into followups(statement_id, followup_id, target_yes, target_no) values (2, 4, 0, 1)")
        .execute(&pool)
        .await?;

    // expect no-followup statement in queue
    let count =
        sqlx::query_scalar!("select count(*) from queue where user_id = 17 and statement_id = 3")
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 0);
    let count =
        sqlx::query_scalar!("select count(*) from queue where user_id = 17 and statement_id = 4")
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 1);

    Ok(())
}

// adding a followup, puts the followup in the queue for it-depends voters
#[sqlx::test]
async fn followup_adds_it_depends_followup_to_queue(pool: SqlitePool) -> sqlx::Result<()> {
    sqlx::query!("insert into users(id, secret) values (17, 'abc')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (2, 'The world is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (3, 'The universe is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (4, 'The world is round.')")
        .execute(&pool)
        .await?;

    //////////////////////////////
    // add it-depends vote
    sqlx::query!("insert into vote_history(user_id, statement_id, vote) values (17, 2, 2)")
        .execute(&pool)
        .await?;

    // add followup
    sqlx::query!("insert into followups(statement_id, followup_id, target_yes, target_no) values (2, 3, 1, 0)")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into followups(statement_id, followup_id, target_yes, target_no) values (2, 4, 0, 1)")
        .execute(&pool)
        .await?;

    // expect both followup statements in queue
    let count =
        sqlx::query_scalar!("select count(*) from queue where user_id = 17 and statement_id = 3")
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 1);
    let count =
        sqlx::query_scalar!("select count(*) from queue where user_id = 17 and statement_id = 4")
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 1);

    Ok(())
}

// subscribing a statement with a followup, puts that followup in the queue for the subscriber
#[sqlx::test]
async fn subscribe_adds_followup_to_queue(pool: SqlitePool) -> sqlx::Result<()> {
    sqlx::query!("insert into users(id, secret) values (17, 'abc')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (2, 'The world is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (3, 'The universe is flat.')")
        .execute(&pool)
        .await?;

    //////////////////////////////
    // add followup
    sqlx::query!("insert into followups(statement_id, followup_id, target_yes, target_no) values (2, 3, 1, 0)")
        .execute(&pool)
        .await?;

    // subscribe
    sqlx::query!("insert into subscriptions(user_id, statement_id) values (17, 2)")
        .execute(&pool)
        .await?;

    // expect followup statement in queue
    let count =
        sqlx::query_scalar!("select count(*) from queue where user_id = 17 and statement_id = 3")
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 1);

    Ok(())
}

// adding a followup, puts the followup in the queue for subscribers
#[sqlx::test]
async fn followup_adds_subscriber_followup_to_queue(pool: SqlitePool) -> sqlx::Result<()> {
    sqlx::query!("insert into users(id, secret) values (17, 'abc')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (2, 'The world is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (3, 'The universe is flat.')")
        .execute(&pool)
        .await?;

    //////////////////////////////
    // subscribe
    sqlx::query!("insert into subscriptions(user_id, statement_id) values (17, 2)")
        .execute(&pool)
        .await?;

    // add followup
    sqlx::query!("insert into followups(statement_id, followup_id, target_yes, target_no) values (2, 3, 1, 0)")
        .execute(&pool)
        .await?;

    // expect followup statement in queue
    let count =
        sqlx::query_scalar!("select count(*) from queue where user_id = 17 and statement_id = 3")
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 1);

    Ok(())
}

// followup is updated, add it to corresponding voters
#[sqlx::test]
async fn followup_updated_yes_voters_to_queue(pool: SqlitePool) -> sqlx::Result<()> {
    sqlx::query!("insert into users(id, secret) values (17, 'abc')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (2, 'The world is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (3, 'The universe is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into followups(statement_id, followup_id, target_yes, target_no) values (2, 3, 0, 1)")
        .execute(&pool)
        .await?;

    //////////////////////////////
    // add yes vote
    sqlx::query!("insert into vote_history(user_id, statement_id, vote) values (17, 2, 1)")
        .execute(&pool)
        .await?;

    // update followup
    sqlx::query!("update followups set target_yes = 1 where statement_id = 2 and followup_id = 3")
        .execute(&pool)
        .await?;

    // expect yes-followup statement in queue
    let count =
        sqlx::query_scalar!("select count(*) from queue where user_id = 17 and statement_id = 3")
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 1);

    Ok(())
}

#[sqlx::test]
async fn followup_updated_no_voters_to_queue(pool: SqlitePool) -> sqlx::Result<()> {
    sqlx::query!("insert into users(id, secret) values (17, 'abc')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (2, 'The world is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into statements(id, text) values (3, 'The universe is flat.')")
        .execute(&pool)
        .await?;
    sqlx::query!("insert into followups(statement_id, followup_id, target_yes, target_no) values (2, 3, 1, 0)")
        .execute(&pool)
        .await?;

    //////////////////////////////
    // add yes vote
    sqlx::query!("insert into vote_history(user_id, statement_id, vote) values (17, 2, -1)")
        .execute(&pool)
        .await?;

    // update followup
    sqlx::query!("update followups set target_no = 1 where statement_id = 2 and followup_id = 3")
        .execute(&pool)
        .await?;

    // expect yes-followup statement in queue
    let count =
        sqlx::query_scalar!("select count(*) from queue where user_id = 17 and statement_id = 3")
            .fetch_one(&pool)
            .await?;
    assert_eq!(count, 1);

    Ok(())
}

// Full Text Search
#[sqlx::test]
async fn maintain_statement_full_text_search_index(pool: SqlitePool) -> sqlx::Result<()> {
    // empty
    let results = sqlx::query("select id, text from statements_fts where text match 'flat'")
        .fetch_all(&pool)
        .await?;
    assert_eq!(results.len(), 0);

    // insert
    sqlx::query!("insert into statements(id, text) values (2, 'The world is flat.')")
        .execute(&pool)
        .await?;

    let results = sqlx::query("select id, text from statements_fts where text match 'flat'")
        .fetch_all(&pool)
        .await?;
    assert_eq!(results.len(), 1);

    // update
    sqlx::query!("update statements set text = 'The world is round.' where id = 2")
        .execute(&pool)
        .await?;

    let count = sqlx::query_scalar!("select count(*) from statements_fts where text match 'flat'")
        .fetch_one(&pool)
        .await?;
    assert_eq!(count, 0);

    let count = sqlx::query_scalar!("select count(*) from statements_fts where text match 'round'")
        .fetch_one(&pool)
        .await?;
    assert_eq!(count, 1);

    // delete
    sqlx::query!("delete from statements where id = 2")
        .execute(&pool)
        .await?;

    let count = sqlx::query_scalar!("select count(*) from statements_fts where text match 'round'")
        .fetch_one(&pool)
        .await?;
    assert_eq!(count, 0);

    Ok(())
}
