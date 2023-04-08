use async_trait::async_trait;

use crate::apikey::{ApiKey, ApiKeyStore};

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
