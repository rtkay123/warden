use std::collections::HashMap;

use serde::Serialize;
use tracing::{debug, instrument};
use warden_infra::postgres::Database;

pub struct TransactionHistoryClient {
    client: sqlx::PgPool,
}

impl TryFrom<HashMap<Database, sqlx::PgPool>> for TransactionHistoryClient {
    type Error = String;

    fn try_from(value: HashMap<Database, sqlx::PgPool>) -> Result<Self, Self::Error> {
        let res = value
            .get(&Database::TransactionHistory)
            .ok_or_else(|| "transaction history is not configured".to_string());
        Ok(Self {
            client: res?.clone(),
        })
    }
}

impl TransactionHistoryClient {
    pub async fn new(client: sqlx::PgPool) -> Result<Self, sqlx::Error> {
        debug!("running migrations");
        sqlx::migrate!("./migrations").run(&client).await?;

        Ok(Self { client })
    }
}

pub trait SaveTransactionHistory {
    fn save_transaction_history(
        &self,
        value: impl Serialize + std::fmt::Debug,
    ) -> impl Future<Output = Result<(), sqlx::Error>>;
}

impl SaveTransactionHistory for TransactionHistoryClient {
    #[instrument(skip(self))]
    async fn save_transaction_history(
        &self,
        value: impl Serialize + std::fmt::Debug,
    ) -> Result<(), sqlx::Error> {
        let a = sqlx::types::Json(value);
        todo!()
    }
}
