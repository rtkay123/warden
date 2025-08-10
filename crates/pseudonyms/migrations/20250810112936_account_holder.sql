create table account_holder (
    source varchar references entity(id),
    destination varchar references account(id),
    cre_dt_tm timestamptz not null,
    primary key (source, destination)
);
