-- all table managed by fastn are stored in fastn schema
CREATE SCHEMA IF NOT EXISTS fastn;


-- Design: https://github.com/FifthTry/ft-sdk/pull/6/
CREATE TABLE IF NOT EXISTS fastn.fastn_user
(
    id       BIGSERIAL primary key,
    name     TEXT NULL,
    username TEXT NULL,
    data     JSONB
);

CREATE TABLE IF NOT EXISTS fastn.fastn_session
(
    id     BIGSERIAL primary key,
    "user" BIGINT,
    data   JSONB,

    constraint fk_tests_students
        foreign key ("user")
            REFERENCES fastn.fastn_user (id)
);



