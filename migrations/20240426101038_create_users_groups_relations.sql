create table users_groups_relations
(
    user_id         bigint             not null,
    group_id        bigint             not null,
    created         timestamptz        not null default now(),
    constraint users_groups_relations_pkey          primary key (user_id, group_id),
    constraint users_groups_relations_user_id_fk    foreign key (user_id) references users (id) on delete cascade,
    constraint users_groups_relations_group_id_fk   foreign key (group_id) references groups (id) on delete cascade
);