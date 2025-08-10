use time::OffsetDateTime;
use tonic::{Request, Response, Status};
use tracing::{Instrument, info_span, instrument};
use warden_core::{
    google,
    pseudonyms::transaction_relationship::{
        CreatePseudonymRequest, mutate_pseudonym_server::MutatePseudonym,
    },
};
use warden_stack::{
    opentelemetry_semantic_conventions::attribute, tracing_opentelemetry::OpenTelemetrySpanExt,
};

use crate::state::AppHandle;

#[tonic::async_trait]
impl MutatePseudonym for AppHandle {
    #[instrument(skip(self, request), err(Debug))]
    async fn create_pseudonym(
        &self,
        request: Request<CreatePseudonymRequest>,
    ) -> Result<Response<google::protobuf::Empty>, Status> {
        let body = request.into_inner();
        let transaction_relationship = body
            .transaction_relationship
            .ok_or_else(|| tonic::Status::data_loss("transaction_relationship"))?;
        let mut tx = self
            .services
            .postgres
            .begin()
            .await
            .map_err(|_e| tonic::Status::internal("database is not ready"))?;

        let span = info_span!("create.pseudonyms.account");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "postgres");
        span.set_attribute(attribute::DB_OPERATION_NAME, "insert");
        span.set_attribute(attribute::DB_QUERY_TEXT, "insert into account");
        span.set_attribute(attribute::DB_COLLECTION_NAME, "account");
        span.set_attribute(
            attribute::DB_QUERY_PARAMETER,
            format!("{}{}", body.creditor_account_id, body.debtor_account_id),
        );

        sqlx::query!(
            "insert into account (id)
            select * from unnest($1::text[])
            on conflict (id) do nothing",
            &[
                body.debtor_account_id.to_string(),
                body.creditor_account_id.to_string()
            ]
        )
        .execute(&mut *tx)
        .instrument(span)
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let span = info_span!("create.pseudonyms.entity");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "postgres");
        span.set_attribute(attribute::DB_OPERATION_NAME, "insert");
        span.set_attribute(attribute::DB_QUERY_TEXT, "insert into entity");
        span.set_attribute(attribute::DB_COLLECTION_NAME, "entity");
        span.set_attribute(
            attribute::DB_QUERY_PARAMETER,
            format!("{}{}", body.creditor_id, body.debtor_id),
        );
        let cre_dt_tm = transaction_relationship.cre_dt_tm.expect("cre_dt_tm");
        let cre_dt_tm = OffsetDateTime::try_from(cre_dt_tm).expect("offset date time conv");

        sqlx::query!(
            "insert into entity (id, cre_dt_tm)
            select * from unnest($1::text[], $2::timestamptz[])
            on conflict (id)
            do update set cre_dt_tm = excluded.cre_dt_tm
            ",
            &[body.creditor_id.to_string(), body.debtor_id.to_string()],
            &[cre_dt_tm, cre_dt_tm]
        )
        .execute(&mut *tx)
        .instrument(span)
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let account_holders = &[
            (
                body.debtor_id.to_string(),
                body.creditor_account_id.to_string(),
            ),
            (
                body.creditor_id.to_string(),
                body.creditor_account_id.to_string(),
            ),
        ];
        let mut deb_holder = vec![];
        let mut cred_holder = vec![];
        let mut dts = vec![];

        account_holders.iter().for_each(|todo| {
            deb_holder.push(todo.0.to_string());
            cred_holder.push(todo.1.to_string());
            dts.push(cre_dt_tm);
        });

        let span = info_span!("create.pseudonyms.account_holder");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "postgres");
        span.set_attribute(attribute::DB_OPERATION_NAME, "insert");
        span.set_attribute(attribute::DB_QUERY_TEXT, "insert into account_holder");
        span.set_attribute(attribute::DB_COLLECTION_NAME, "account_holder");

        sqlx::query!(
            "insert into account_holder (source, destination, cre_dt_tm)
            select * from unnest($1::text[], $2::text[], $3::timestamptz[])
            on conflict (source, destination)
            do update set cre_dt_tm = excluded.cre_dt_tm
            ",
            &deb_holder,
            &cred_holder,
            &dts
        )
        .execute(&mut *tx)
        .instrument(span)
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let latlng: Option<(f64, f64)> = transaction_relationship
            .latlng
            .map(|value| (value.latitude, value.longitude));

        let span = info_span!("create.pseudonyms.transaction_relationship");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "postgres");
        span.set_attribute(attribute::DB_OPERATION_NAME, "insert");
        span.set_attribute(attribute::DB_COLLECTION_NAME, "transaction_relationship");
        span.set_attribute(
            attribute::DB_QUERY_TEXT,
            "insert into transaction_relationship",
        );

        let amt = transaction_relationship
            .amt
            .ok_or_else(|| tonic::Status::data_loss("amt"))?;

        sqlx::query!(
            "
            insert into transaction_relationship (
                source,
                destination,
                amt_unit,
                amt_ccy,
                amt_nanos,
                cre_dt_tm,
                end_to_end_id,
                msg_id,
                pmt_inf_id,
                tx_tp,
                lat,
                lon,
                tx_sts
            )
            values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ",
            transaction_relationship.from,
            transaction_relationship.to,
            amt.units,
            amt.currency_code,
            amt.nanos,
            cre_dt_tm,
            transaction_relationship.end_to_end_id,
            transaction_relationship.msg_id,
            transaction_relationship.pmt_inf_id,
            transaction_relationship.tx_tp,
            latlng.map(|lat| lat.0),
            latlng.map(|lat| lat.1),
            transaction_relationship.tx_sts,
        )
        .execute(&mut *tx)
        .instrument(span)
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let span = info_span!("transaction.commit");
        span.set_attribute(attribute::DB_SYSTEM_NAME, "postgres");
        span.set_attribute(attribute::DB_OPERATION_NAME, "commit");

        tx.commit()
            .instrument(span)
            .await
            .map_err(|_e| tonic::Status::internal("database is not ready"))?;
        Ok(Response::new(google::protobuf::Empty::default()))
    }
}
