use anyhow::{anyhow, Result};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use async_trait::async_trait;

// we use a static salt, since we only hash part of the actual API key
static SALT: &str = "staticsalt";
// how many characters to use from the original API key
static KEY_PART_LEN: usize = 12;

/// Returns a hash based on a trimmed(!) API key
pub fn api_key_partial_hash(s: &str) -> anyhow::Result<String> {
    assert!(
        s.len() > KEY_PART_LEN,
        "Key is below minimum size. Key: {}. Size: {}",
        s.len(),
        KEY_PART_LEN
    );
    let s = &s[0..KEY_PART_LEN];
    let salt = match SaltString::encode_b64(SALT.as_bytes()) {
        Ok(salt) => Ok(salt),
        Err(err) => Err(anyhow!(err)),
    }?;
    let a2 = Argon2::default();
    let hashed = a2.hash_password(s.as_bytes(), &salt);
    // Ok(hashed?.to_string())
    let hashed = match hashed {
        Ok(hash) => Ok(hash),
        Err(err) => Err(anyhow!(err)),
    }?;
    Ok(hashed.to_string())
}

/// Contains a not yet persistet api key
#[derive(Debug)]
pub enum TransientApiKey {
    /// A key that has not yet been hashed
    Raw(String),
    /// A key that has already been hashed
    #[allow(dead_code)]
    Hashed(String),
}

/// Represents a persistent API key
#[derive(serde::Serialize, sqlx::FromRow, Clone, Debug, Eq, PartialEq)]
pub struct ApiKey {
    pub id: i64,
    pub hash: String,
    pub note: Option<String>,
}

/// Defines which methods have to be implemented on the store to work with ApiKey
#[async_trait]
pub trait ApiKeyStore {
    /// Store the item inside the particular DB
    async fn store(&mut self, item: &ApiKey) -> anyhow::Result<ApiKey>;
    /// Retrieve by id
    async fn by_id(&self, id: i64) -> anyhow::Result<Option<ApiKey>>;
    /// Retrieve by TransientApiKey
    async fn by_hash(&self, hash: &str) -> anyhow::Result<Option<ApiKey>>;
}

impl ApiKey {
    /// Create a new key inside the DB
    pub async fn create<S: ToString, Store: ApiKeyStore>(
        store: &mut Store,
        tkey: &TransientApiKey,
        note: Option<S>,
    ) -> Result<Self> {
        let hashed: String = match tkey {
            TransientApiKey::Raw(raw_api_key) => api_key_partial_hash(raw_api_key)?,
            TransientApiKey::Hashed(hashed) => hashed.into(),
        };
        let v = Self {
            id: 0,
            hash: hashed,
            note: note.map(|s| s.to_string()),
        };
        store.store(&v).await
    }
    /// Read from DB by passing in key value
    pub async fn by_transient<Store: ApiKeyStore>(
        store: &Store,
        tkey: &TransientApiKey,
    ) -> Result<Option<Self>> {
        let hash : String = match tkey {
            TransientApiKey::Raw(raw_api_key) => api_key_partial_hash(raw_api_key)?,
            TransientApiKey::Hashed(hash) => hash.into(),
        };
        store.by_hash(hash.as_str()).await
    }
    /// Read from DB or create the key
    pub async fn get_or_create<S: ToString, Store: ApiKeyStore>(
        store: &mut Store,
        tkey: &TransientApiKey,
        note: Option<S>,
    ) -> Result<Self> {
        let maybe_key = Self::by_transient(store, tkey).await?;
        Ok(match maybe_key {
            Some(key) => key,
            None => Self::create(store, tkey, note).await?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::InMemoryStore;

    #[async_trait::async_trait]
    impl ApiKeyStore for InMemoryStore<String, ApiKey> {
        async fn store(&mut self, item: &ApiKey) -> anyhow::Result<ApiKey> {
            self.values.insert(item.hash.to_owned(), item.to_owned());
            Ok(self.values.get(&item.hash).unwrap().to_owned())
        }
        async fn by_id(&self, _id: i64) -> anyhow::Result<Option<ApiKey>> {
            todo!()
        }
        async fn by_hash(&self, hash: &str) -> anyhow::Result<Option<ApiKey>> {
            Ok(self.values.get(hash.into()).cloned())
        }
    }


    #[tokio::test]
    async fn test_api_key_from_unhashed() -> Result<()> {
        let mut db = InMemoryStore::new();
        let unhashed: String = "abcdefgabcdefgabcdefg".into();
        let tkey = TransientApiKey::Raw(unhashed.to_owned());
        let note = Some("".to_string());
        let key = ApiKey::create(&mut db, &tkey, note.to_owned()).await?;
        assert_ne!(key.hash, unhashed);
        assert_eq!(key.note, note);
        assert_eq!(key, ApiKey::by_transient(&mut db, &tkey).await?.unwrap());
        Ok(())
    }

    #[tokio::test]
    async fn test_api_key_from_hashed() -> Result<()> {
        let mut db = InMemoryStore::new();
        let hashed: String = "abcdefgabcdefgabcdefg".into();
        let tkey = TransientApiKey::Hashed(hashed.to_owned());
        let note = Some("".to_string());
        let key = ApiKey::get_or_create(&mut db, &tkey, note.to_owned()).await?;
        assert_eq!(key.hash, hashed);
        assert_eq!(key.note, note);
        assert_eq!(key, ApiKey::by_transient(&mut db, &tkey).await?.unwrap());
        Ok(())
    }
}
