create table pacs002 (
    id uuid primary key,
    document jsonb not null,
    created_at timestamptz default now(),
    processed boolean,
    
    message_id text generated always as (
        document->'f_i_to_f_i_pmt_sts_rpt'->'grp_hdr'->>'msg_id'
    ) stored,

    end_to_end_id text generated always as (
        document->'f_i_to_f_i_pmt_sts_rpt'->'tx_inf_and_sts'->0->>'orgnl_end_to_end_id'
    ) stored,

    constraint unique_msgid_e2eid_pacs002 unique (message_id, end_to_end_id),

    constraint message_id_not_null check (message_id is not null),
    constraint end_to_end_id_not_null check (end_to_end_id is not null)
);

create table pacs008 (
    id uuid primary key,
    document jsonb not null,
    created_at timestamptz default now(),
    processed boolean,

    message_id text generated always as (
        document->'f_i_to_f_i_cstmr_cdt_trf'->'grp_hdr'->>'msg_id'
    ) stored,

    end_to_end_id text generated always as (
        document->'f_i_to_f_i_cstmr_cdt_trf'->'cdt_trf_tx_inf'->0->'pmt_id'->>'end_to_end_id'
    ) stored,

    constraint unique_msgid_e2eid_pacs008 unique (message_id, end_to_end_id),
    constraint message_id_not_null check (message_id is not null),
    constraint end_to_end_id_not_null check (end_to_end_id is not null)
);
