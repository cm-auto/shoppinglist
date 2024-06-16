create table entries
(
    id              bigserial primary key,
    product         varchar(100)    not null,
    amount          real            not null,
    unit            varchar(30)     not null,
    note            varchar(200)    ,
    created         timestamptz     not null default now(),
    updated         timestamptz     null,
    bought          timestamptz     null,
    user_id         bigint          not null,
    group_id        bigint          ,
    constraint entries_user_id_fk    foreign key (user_id) references users (id) on delete cascade,
    constraint entries_group_id_fk   foreign key (group_id) references groups (id) on delete cascade
);

create trigger set_updated_on_entries
before update on entries
for each row
execute procedure trigger_set_updated();