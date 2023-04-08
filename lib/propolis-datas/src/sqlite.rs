use async_trait::async_trait;

use crate::{
    apikey::{ApiKey, ApiKeyStore},
    statement::{StatementFlag, StatementFlagStore},
};

#[async_trait]
impl ApiKeyStore for sqlx::SqlitePool {
    async fn store(&mut self, item: &ApiKey) -> anyhow::Result<ApiKey> {
        sqlx::query!(
            "INSERT INTO api_keys (hash, note) VALUES (?, ?)",
            item.hash,
            item.note
        )
        .execute(self as &sqlx::SqlitePool)
        .await?;
        let r = self.by_hash(item.hash.as_str()).await?;
        r.ok_or(anyhow::anyhow!("Unable to retrieve just stored value"))
    }
    async fn by_id(&self, id: i64) -> anyhow::Result<Option<ApiKey>> {
        Ok(sqlx::query_as!(
            ApiKey,
            "SELECT id, hash, note FROM api_keys WHERE id = ?",
            id
        )
        .fetch_optional(self)
        .await?)
    }
    async fn by_hash(&self, hash: &str) -> anyhow::Result<Option<ApiKey>> {
        Ok(sqlx::query_as!(
            ApiKey,
            "SELECT id, hash, note FROM api_keys WHERE hash = ?",
            hash
        )
        .fetch_optional(self)
        .await?)
    }
}

#[async_trait]
impl StatementFlagStore for sqlx::SqlitePool {
    async fn store(&mut self, item: &StatementFlag) -> anyhow::Result<StatementFlag> {
        let state = item.state as i64;
        let categories = serde_json::to_string(&item.categories)?;

        sqlx::query!(
            "INSERT INTO statement_flags (statement_id, state, categories)
 VALUES (?, ?, ?)",
            item.statement_id,
            state,
            categories,
        )
        .execute(self as &sqlx::SqlitePool)
        .await?;
        let r = self.by_statement_id(item.statement_id).await?;
        r.ok_or(anyhow::anyhow!("Unable to retrieve just stored value"))
    }
    async fn by_statement_id(&self, id: i64) -> anyhow::Result<Option<StatementFlag>> {
        Ok(sqlx::query_as!(
            StatementFlag,
            "SELECT statement_id, state, categories, created FROM statement_flags WHERE statement_id = ?",
            id
        )
           .fetch_optional(self)
           .await?)
    }
    async fn update(&self, item: &StatementFlag) -> anyhow::Result<()> {
        let state = item.state as i64;
        let categories = serde_json::to_string(&item.categories)?;
        sqlx::query!(
            "UPDATE statement_flags
SET state = ?, categories = ?
WHERE statement_id = ?",
            state,
            categories,
            item.statement_id,
        )
        .execute(self as &sqlx::SqlitePool)
        .await?;
        Ok(())
    }
}
