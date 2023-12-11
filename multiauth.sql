-- TODO: move to some place else

CREATE TABLE IF NOT EXISTS fastn_user (
    id UUID PRIMARY KEY,
    email VARCHAR(254) NOT NULL,
    username TEXT,
    password TEXT,
    name TEXT NOT NULL,
    created_on TIMESTAMP
);

CREATE TABLE IF NOT EXISTS fastn_session (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES fastn_user(id),
    created_on TIMESTAMP
);

CREATE TABLE IF NOT EXISTS fastn_oauthtoken (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES fastn_session(id) ON DELETE CASCADE,
    token text NOT NULL,
    provider text NOT NULL,
    created_on TIMESTAMP DEFAULT now()
);


ALTER TABLE fastn_user ALTER COLUMN created_on SET DEFAULT now();
ALTER TABLE fastn_session ALTER COLUMN created_on SET DEFAULT now();
ALTER TABLE fastn_user ADD UNIQUE (email);
ALTER TABLE fastn_user ADD UNIQUE (username);
