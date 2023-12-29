-- https://dba.stackexchange.com/a/165923
CREATE EXTENSION citext;
CREATE DOMAIN email AS citext
    CHECK (value ~ '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$');


-- registered user
CREATE TABLE IF NOT EXISTS fastn_user (
    id SERIAL PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- logged in user session store
CREATE TABLE IF NOT EXISTS fastn_session (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES fastn_user(id) ON DELETE CASCADE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE default NOW() NOT NULL
);

-- token from oauth apps
-- TODO: handle expiration of tokens
CREATE TABLE IF NOT EXISTS fastn_oauthtoken (
    id SERIAL PRIMARY KEY,
    session_id INTEGER REFERENCES fastn_session(id) ON DELETE CASCADE NOT NULL,
    token TEXT NOT NULL,
    provider TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- user emails
CREATE TABLE IF NOT EXISTS fastn_user_email (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES fastn_user(id) ON DELETE CASCADE NOT NULL,
    email email NOT NULL UNIQUE,
    verified BOOLEAN DEFAULT FALSE NOT NULL,
    "primary" BOOLEAN DEFAULT FALSE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- email confirmations. Can't log in without confirming email first
create table if not exists fastn_email_confirmation(
    id SERIAL PRIMARY KEY,
    email_id INTEGER REFERENCES fastn_user_email(id) ON DELETE CASCADE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    sent_at TIMESTAMP WITH TIME ZONE NOT NULL, -- to check expiration
    "key" TEXT UNIQUE NOT NULL -- for verification
);
