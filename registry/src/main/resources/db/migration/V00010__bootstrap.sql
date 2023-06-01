create table tailcall_main_db.blueprint_spec
(
    id               int auto_increment primary key,
    digest_hex       char(64)                            not null unique,
    digest_alg       enum ('SHA-256')                    not null,
    blueprint        mediumblob                          not null,
    blueprint_format enum ('json')                       not null,
    created          timestamp default CURRENT_TIMESTAMP not null,
    dropped          timestamp                           null
);

create index blueprints_digest_index on tailcall_main_db.blueprint_spec (digest_hex(64));
