use anyhow::Result;
use sqlx::PgPool;
use tonic::{Code, IntoRequest};
use warden_core::pseudonyms::transaction_relationship::CreatePseudonymRequest;

use crate::helpers::TestApp;

#[sqlx::test]
async fn data_loss_tr(pool: PgPool) -> Result<()> {
    let mut app = TestApp::new(pool).await;

    let user_request = CreatePseudonymRequest::default();

    let response = app
        .mutate
        .create_pseudonym(user_request.into_request())
        .await;

    dbg!(&response);
    assert!(response.is_err_and(|value| { value.code() == Code::DataLoss }));

    Ok(())
}
