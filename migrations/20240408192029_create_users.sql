create table users
(
    id              bigserial primary key,
    username        varchar(30) unique not null check (username ~ '^[A-Za-z_][A-Za-z0-9_-]*$'),
    password        varchar(80)        not null,
    display_name    varchar(80)        not null,
    created         timestamptz        not null default now(),
    updated         timestamptz            null 
);

create trigger set_updated_on_users
before update on users
for each row
execute procedure trigger_set_updated();