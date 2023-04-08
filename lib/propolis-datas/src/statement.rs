use async_trait::async_trait;
use num_derive::FromPrimitive;

#[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct FlagCategory {
    /// Name of the flag. e.g. "hate"
    pub name: String,
    /// If true the flag is active, else it is not
    pub value: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum FlagCategoryContainer {
    Empty,
    Vec(Vec<FlagCategory>),
}

/// Describes the flagging status of a statement through e.g. a moderation API
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, Copy, Clone, FromPrimitive)]
pub enum StatementFlagState {
    /// No flags
    Clear = 0,
    /// Might be flagged and requires a thorough check
    MaybeFlagged = 1,
    /// Actually flagged with one or more flags
    Flagged = 2,
}

#[derive(serde::Serialize, sqlx::FromRow, Clone)]
pub struct StatementFlag {
    pub statement_id: i64,
    #[sqlx(try_from = "i64")]
    pub state: StatementFlagState,
    #[sqlx(try_from = "String")]
    pub categories: FlagCategoryContainer,
    pub created: i64,
}

// impl TryFrom<i64> for StatementFlagState {
//     type Error = anyhow::Error;

//     fn try_from(value: i64) -> Result<Self, Self::Error> {
//         Ok(match value {
//             1 => StatementFlagState::MaybeFlagged,
//             2 => StatementFlagState::Flagged,
//             _ => StatementFlagState::Clear,
//         })
//     }
// }
impl From<i64> for StatementFlagState {
    fn from(value: i64) -> Self {
        match value {
            1 => StatementFlagState::MaybeFlagged,
            2 => StatementFlagState::Flagged,
            _ => StatementFlagState::Clear,
        }
    }
}

impl From<String> for FlagCategoryContainer {
    fn from(value: String) -> Self {
        serde_json::from_str(value.as_str()).unwrap_or(FlagCategoryContainer::Empty)
    }
}

/// Defines which methods have to be implemented on the store to work with StatementFlag
#[async_trait]
pub trait StatementFlagStore {
    /// Store the item inside the particular DB
    async fn store(&mut self, item: &StatementFlag) -> anyhow::Result<StatementFlag>;
    /// Retrieve by id
    async fn by_statement_id(&self, id: i64) -> anyhow::Result<Option<StatementFlag>>;
    /// Update
    async fn update(&self, flag: &StatementFlag) -> anyhow::Result<()>;
}

impl StatementFlag {
    /// Create a new instance inside the DB
    pub async fn create<Store: StatementFlagStore>(
        store: &mut Store,
        statement_id: i64,
        state: StatementFlagState,
        categories: FlagCategoryContainer,
    ) -> anyhow::Result<Self> {
        let v = Self {
            statement_id,
            state,
            categories,
            created: 0,
        };
        store.store(&v).await
    }

    /// Read from DB by passing in statement id
    pub async fn by_statement_id<Store: StatementFlagStore>(
        store: &Store,
        statement_id: i64,
    ) -> anyhow::Result<Option<Self>> {
        store.by_statement_id(statement_id).await
    }

    pub async fn update<Store: StatementFlagStore>(&self, store: &Store) -> anyhow::Result<()> {
        store.update(self).await
    }
    // /// Read from DB or create
    // pub async fn get_or_create<S: ToString, Store: StatementFlagStore>(
    //     store: &mut Store,
    //     statement_id: i64,
    //     note: Option<S>,
    // ) -> anyhow::Result<Self> {
    //     let maybe_key = Self::by_transient(store, tkey).await?;
    //     Ok(match maybe_key {
    //         Some(key) => key,
    //         None => Self::create(store, tkey, note).await?,
    //     })
    // }
}
