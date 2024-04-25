CREATE TABLE IF NOT EXISTS fastn_user
(
    id       INTEGER primary key,
    name     TEXT NULL,
    username TEXT NULL,
    data     JSONB -- this stores ft_sdk::auth::UserData
) STRICT;

CREATE TABLE IF NOT EXISTS fastn_session
(
    id   INTEGER primary key,
    uid  BIGINT NULL,
    data JSONB, -- this is the session data only

    CONSTRAINT fk_fastn_user
        FOREIGN KEY (uid)
            REFERENCES fastn_user (id)
) STRICT;




