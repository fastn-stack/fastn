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
    id   BIGSERIAL primary key,
    uid  BIGINT,
    data JSONB,

    CONSTRAINT fk_tests_students
        FOREIGN KEY (uid)
            REFERENCES fastn.fastn_user (id)
);




