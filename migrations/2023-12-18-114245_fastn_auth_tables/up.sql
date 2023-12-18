-- https://dba.stackexchange.com/a/165923
CREATE EXTENSION citext;
CREATE DOMAIN email AS citext
    CHECK (value ~ '^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$');


-- registered user
CREATE TABLE IF NOT EXISTS fastn_user (
    id SERIAL PRIMARY KEY,
    username TEXT UNIQUE,
    password TEXT,
    name TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- logged in user session store
CREATE TABLE IF NOT EXISTS fastn_session (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES fastn_user(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE default NOW()
);

-- token from oauth apps
-- TODO: handle expiration of tokens
CREATE TABLE IF NOT EXISTS fastn_oauthtoken (
    id SERIAL PRIMARY KEY,
    session_id INTEGER REFERENCES fastn_session(id) ON DELETE CASCADE,
    token TEXT NOT NULL,
    provider TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- user emails
CREATE TABLE IF NOT EXISTS fastn_user_email (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES fastn_user(id) ON DELETE CASCADE,
    email email NOT NULL UNIQUE,
    verified BOOLEAN DEFAULT FALSE,
    "primary" BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- email confirmations. Can't log in without confirming email first
create table if not exists fastn_email_confirmation(
    id SERIAL PRIMARY KEY,
    email_id INTEGER REFERENCES fastn_user_email(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    sent_at TIMESTAMP WITH TIME ZONE, -- to check expiration
    "key" TEXT UNIQUE -- for verification
);

