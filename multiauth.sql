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
    id SERIAL PRIMARY KEY,
    user_id UUID REFERENCES fastn_user(id),
    created_on TIMESTAMP
);


ALTER TABLE fastn_user ALTER COLUMN created_on SET DEFAULT now();
