create table routing (
    id uuid primary key,
    configuration jsonb not null
);

create index idx_active_routing on routing using gin ((configuration->'active'));
