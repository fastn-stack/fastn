-- https://dba.stackexchange.com/a/165923
CREATE EXTENSION citext;

CREATE DOMAIN email AS citext
    CHECK (value ~ '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$');


-- registered user
CREATE TABLE IF NOT EXISTS fastn_user (
    id BIGSERIAL PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    email email NOT NULL UNIQUE, -- de-normalised data to avoid joins
    verified_email BOOLEAN DEFAULT FALSE NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);

-- logged in user session store
-- right now, we do not have a generalised session store like django's
-- in future, this table will be dissolved into that session table
CREATE TABLE IF NOT EXISTS fastn_auth_session (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT REFERENCES fastn_user(id) ON DELETE CASCADE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);

-- token from oauth apps
-- TODO: handle expiration of tokens
CREATE TABLE IF NOT EXISTS fastn_oauthtoken (
    id BIGSERIAL PRIMARY KEY,
    session_id BIGINT REFERENCES fastn_auth_session(id) ON DELETE CASCADE NOT NULL,
    token TEXT NOT NULL,
    provider TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);

-- user emails
CREATE TABLE IF NOT EXISTS fastn_user_email (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT REFERENCES fastn_user(id) ON DELETE CASCADE NOT NULL,
    email email NOT NULL UNIQUE,
    verified BOOLEAN DEFAULT FALSE NOT NULL,
    "primary" BOOLEAN DEFAULT FALSE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL
);

-- email confirmations. Can't log in without confirming email first
create table if not exists fastn_email_confirmation(
    id BIGSERIAL PRIMARY KEY,
    email_id BIGINT REFERENCES fastn_user_email(id) ON DELETE CASCADE NOT NULL,
    session_id BIGINT REFERENCES fastn_auth_session(id) ON DELETE CASCADE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    sent_at TIMESTAMP WITH TIME ZONE NOT NULL, -- to check expiration
    "key" TEXT UNIQUE NOT NULL -- for verification
);
