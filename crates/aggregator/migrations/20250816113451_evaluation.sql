-- Add migration script here
create table evaluation (
    id uuid primary key,
    document jsonb not null,
    created_at timestamptz default now()
);
