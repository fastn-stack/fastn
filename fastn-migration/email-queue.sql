CREATE TABLE IF NOT EXISTS fastn_email_queue
(
    id          BIGSERIAL PRIMARY KEY,
    to_email    TEXT NOT NULL,
    to_name     TEXT NOT NULL,
    subject     TEXT NOT NULL,
    body        TEXT NOT NULL,
    retry_count INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at  TIMESTAMP WITH TIME ZONE NOT NULL,
    mkind       TEXT NOT NULL,
    status      TEXT NOT NULL
);
