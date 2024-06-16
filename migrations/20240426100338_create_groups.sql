create table groups
(
    id              bigserial          primary key,
    name            varchar(80)        not null,
    created         timestamptz        not null default now(),
    updated         timestamptz            null 
);

create trigger set_updated_on_groups
before update on groups
for each row
execute procedure trigger_set_updated();