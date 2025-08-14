create table rule (
    uuid uuid primary key,
    configuration jsonb not null,
    id text generated always as (
        configuration->>'id'
    ) stored,
    version text generated always as (
        configuration->>'version'
    ) stored,
    unique (id, version)
);
