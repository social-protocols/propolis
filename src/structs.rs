use serde::Serialize;

#[derive(Serialize, sqlx::FromRow)]
pub struct Statement {
    pub id: i64,
    pub text: String,
}
