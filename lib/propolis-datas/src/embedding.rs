use async_trait::async_trait;

/// Contains an embedding result
#[derive(Clone, Debug)]
pub struct Embedding {
    pub statement_id: i64,
    pub data: Vec<f64>,
    pub prompt_tokens: i64,
    pub api_key_id: i64,
}

/// Defines which methods have to be implemented on the store to work with Embedding
#[async_trait]
pub trait EmbeddingStore {
    /// Store the item inside the particular DB
    async fn store(&mut self, item: &Embedding) -> anyhow::Result<Embedding>;
    /// Retrieve by id
    async fn by_statement_id(&self, id: i64) -> anyhow::Result<Option<Embedding>>;
}

impl Embedding {
    pub async fn create<Store: EmbeddingStore>(
        store: &mut Store,
        statement_id: i64,
        data: Vec<f64>,
        prompt_tokens: i64,
        api_key_id: i64,
    ) -> anyhow::Result<Embedding> {
        store.store(&Embedding {
            statement_id,
            data,
            prompt_tokens,
            api_key_id,
        }).await
    }
}
