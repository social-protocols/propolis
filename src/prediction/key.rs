use anyhow::{anyhow, Result};
use async_trait::async_trait;
use db::InMemoryStore;
use pwhash::bcrypt::hash;
use sqlx::SqlitePool;

/// Contains a not yet persistet api key
pub enum TransientApiKey {
    /// A key that has not yet been hashed
    Raw(String),
    /// A key that has already been hashed
    Hashed(String),
}

#[derive(serde::Serialize, sqlx::FromRow, Clone, Debug, Eq, PartialEq)]
pub struct Test {}

/// Represents a persistent API key
#[derive(serde::Serialize, sqlx::FromRow, Clone, Debug, Eq, PartialEq)]
pub struct ApiKey {
    pub id: i64,
    pub hash: String,
    pub note: Option<String>,
}

#[async_trait]
pub trait TestStore {
    async fn store(&mut self) -> anyhow::Result<()>;
}

/// Defines which methods have to be implemented on the store to work with ApiKey
#[async_trait]
pub trait ApiKeyStore {
    /// Store the item inside the particular DB
    async fn store(&mut self, item: &ApiKey) -> anyhow::Result<ApiKey>;
    /// Retrieve the item from the particular DB
    async fn get_by_id(&self, id: i64) -> anyhow::Result<Option<ApiKey>>;
    async fn from_transient(&self, tkey: &TransientApiKey) -> anyhow::Result<Option<ApiKey>>;
}

#[async_trait]
impl ApiKeyStore for SqlitePool {
    async fn store(&mut self, item: &ApiKey) -> anyhow::Result<ApiKey> {
        let id = sqlx::query!(
            "INSERT INTO api_keys (hash, note) VALUES (?, ?)",
            item.hash,
            item.note
        )
        .execute(self as &SqlitePool)
        .await?
        .last_insert_rowid();
        let r = self.get_by_id(id).await?;
        r.ok_or(anyhow!("Unable to retrieve just stored value"))
    }
    async fn get_by_id(&self, id: i64) -> anyhow::Result<Option<ApiKey>> {
        Ok(sqlx::query_as!(
            ApiKey,
            "SELECT id, hash, note FROM api_keys WHERE id = ?",
            id
        )
        .fetch_optional(self)
        .await?)
    }
    async fn from_transient(&self, tkey: &TransientApiKey) -> anyhow::Result<Option<ApiKey>> {
        let hashed: String = match tkey {
            TransientApiKey::Raw(raw_api_key) => hash(raw_api_key)?,
            TransientApiKey::Hashed(hashed) => hashed.into(),
        };
        Ok(sqlx::query_as!(
            ApiKey,
            "SELECT id, hash, note FROM api_keys WHERE hash = ?",
            hashed
        )
        .fetch_optional(self)
        .await?)
    }
}

impl ApiKey {
    /// Create a new key inside the DB
    pub async fn create<S: ToString, Store: AsMut<dyn ApiKeyStore>>(
        store: &mut Store,
        tkey: &TransientApiKey,
        note: Option<S>,
    ) -> Result<Self> {
        let hashed: String = match tkey {
            TransientApiKey::Raw(raw_api_key) => hash(raw_api_key)?,
            TransientApiKey::Hashed(hashed) => hashed.into(),
        };
        let v = Self {
            id: 0,
            hash: hashed,
            note: note.map(|s| s.to_string()),
        };
        store.as_mut().store(&v).await
    }

    /// Read from DB by passing in key id
    pub async fn from_id<Store: AsRef<dyn ApiKeyStore>>(
        store: &Store,
        id: i64,
    ) -> Result<Option<Self>> {
        store.as_ref().get_by_id(id).await
    }

    /// Read from DB by passing in key value
    pub async fn from_transient<Store: AsRef<dyn ApiKeyStore>>(
        store: &Store,
        tkey: &TransientApiKey,
    ) -> Result<Option<Self>> {
        store.as_ref().from_transient(tkey).await
    }
}

#[async_trait::async_trait]
impl ApiKeyStore for db::InMemoryStore<ApiKey> {
    async fn store(&mut self, item: &ApiKey) -> anyhow::Result<ApiKey> {
        let k = self.values.len() as i64;
        self.values.insert(k, item.to_owned());
        Ok(self.values.get(&k).unwrap().to_owned())
    }
    async fn get_by_id(&self, id: i64) -> anyhow::Result<Option<ApiKey>> {
        Ok(self.values.get(&id).cloned())
    }
    async fn from_transient(&self, _tkey: &TransientApiKey) -> anyhow::Result<Option<ApiKey>> {
        todo!();
    }
}

impl<'a> AsMut<dyn ApiKeyStore + 'a> for InMemoryStore<ApiKey> {
    fn as_mut(&mut self) -> &mut (dyn ApiKeyStore + 'a) {
        self
    }
}

impl<'a> AsRef<dyn ApiKeyStore + 'a> for InMemoryStore<ApiKey> {
    fn as_ref(&self) -> &(dyn ApiKeyStore + 'a) {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::InMemoryStore;

    #[tokio::test]
    async fn test_api_key_from_unhashed() -> Result<()> {
        let mut db = InMemoryStore::new();
        let unhashed: String = "abcd".into();
        let tkey = TransientApiKey::Raw(unhashed.to_owned());
        let note = Some("".to_string());
        let key = ApiKey::create(&mut db, &tkey, note.to_owned()).await?;
        assert_ne!(key.hash, unhashed);
        assert_eq!(key.note, note);
        assert_eq!(key, ApiKey::from_id(&mut db, key.id).await?.unwrap());
        Ok(())
    }

    #[tokio::test]
    async fn test_api_key_from_hashed() -> Result<()> {
        let mut db = InMemoryStore::new();
        let hashed: String = "abcd".into();
        let tkey = TransientApiKey::Hashed(hashed.to_owned());
        let note = Some("".to_string());
        let key = ApiKey::create(&mut db, &tkey, note.to_owned()).await?;
        assert_eq!(key.hash, hashed);
        Ok(())
    }
}
