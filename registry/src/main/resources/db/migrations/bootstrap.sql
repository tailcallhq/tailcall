drop
    database if exists tailcall_main_db;

create
    database tailcall_main_db;


create table tailcall_main_db.blueprint_spec
(
    id               int auto_increment primary key,
    digest_hex       char(64)                            not null unique,
    digest_alg       varchar(8)                          not null,
    blueprint        blob                                not null,
    blueprint_format enum ('json')                       not null,
    created          timestamp default CURRENT_TIMESTAMP not null,
    dropped          timestamp                           null
);
create index blueprints_digest_index on tailcall_main_db.blueprint_spec (digest_hex(64));
