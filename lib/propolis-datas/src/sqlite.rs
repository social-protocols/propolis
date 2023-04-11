use async_trait::async_trait;
use sqlx::Row;
use tracing::debug;

use crate::{
    apikey::{ApiKey, ApiKeyStore},
    embedding::{Embedding, EmbeddingStore},
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
        let r = StatementFlagStore::by_statement_id(self, item.statement_id).await?;
        r.ok_or(anyhow::anyhow!("Unable to retrieve just stored value"))
    }
    async fn by_statement_id(&self, id: i64) -> anyhow::Result<Option<StatementFlag>> {
        // Use an intermediary struct since FromRow and other sqlx gimmics do not work
        // FIXME: Try again with sqlx 0.7 and its #[sqlx(try_from)] macro
        struct Row {
            pub statement_id: i64,
            pub state: i64,
            pub categories: String,
            pub created: i64,
        }
        let row = sqlx::query_as!(
            Row,
            "SELECT statement_id, state, categories, created
FROM statement_flags
WHERE statement_id = ?",
            id
        )
        .fetch_optional(self)
        .await?;
        Ok(row.map(|row| StatementFlag {
            statement_id: row.statement_id,
            state: row.state.into(),
            categories: row.categories.into(),
            created: row.created,
        }))
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

#[async_trait]
impl EmbeddingStore for sqlx::SqlitePool {
    async fn store(&mut self, item: &Embedding) -> anyhow::Result<Embedding> {
        let data_json = serde_json::to_string(&item.data)?;
        debug!("Embedding JSON data:");
        debug!("==========");
        debug!("{}", data_json);
        debug!("==========");
        sqlx::query(
            "INSERT INTO statement_embeddings (statement_id, data, prompt_tokens, api_key_id)
 VALUES (?, vector_to_blob(vector_from_json(?)), ?, ?)",
        )
        .bind(item.statement_id)
        .bind(data_json)
        .bind(item.prompt_tokens)
        .bind(item.api_key_id)
        .execute(self as &sqlx::SqlitePool)
        .await?;
        let r = EmbeddingStore::by_statement_id(self, item.statement_id).await?;
        r.ok_or(anyhow::anyhow!("Unable to retrieve just stored value"))
    }
    async fn by_statement_id(&self, id: i64) -> anyhow::Result<Option<Embedding>> {
        let row = sqlx::query(
            "SELECT statement_id, vector_to_json(vector_from_blob(data)), prompt_tokens, api_key_id
FROM statement_embeddings
WHERE statement_id = ?",
        )
        .bind(id)
        .fetch_optional(self)
        .await?;
        Ok(row.map(|row| Embedding {
            statement_id: row.try_get(0).expect("No id"),
            data: serde_json::from_str(row.try_get(1).expect("No data")).expect("Unable to parse data as json"),
            prompt_tokens: row.try_get(2).expect("No prompt_tokens"),
            api_key_id: row.try_get(3).expect("No api key"),
        }))
    }
}
