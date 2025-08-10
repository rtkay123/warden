create table transaction_relationship (
    source varchar references account(id),
    destination varchar references account(id),
    amt_unit bigint not null,
    amt_ccy varchar(3) not null,
    amt_nanos integer not null,
    cre_dt_tm timestamptz not null,
    end_to_end_id varchar not null check (trim(end_to_end_id) <> ''),
    msg_id varchar not null check (trim(msg_id) <> ''),
    pmt_inf_id varchar not null check (trim(pmt_inf_id) <> ''),
    tx_tp varchar not null check (trim(tx_tp) <> ''),
    lat float8,
    lon float8,
    tx_sts varchar,
    primary key (msg_id, end_to_end_id, tx_tp, pmt_inf_id)
);

create index idx_transaction_status_range
  on transaction_relationship (tx_tp, tx_sts, cre_dt_tm desc);

create index idx_transaction_e2eid_tp_sts
  on transaction_relationship (end_to_end_id, tx_tp, tx_sts);

create index idx_transaction_accc_only
  on transaction_relationship (cre_dt_tm)
  where tx_sts = 'ACCC';

create index idx_transaction_source_time
  on transaction_relationship (source, cre_dt_tm desc);
