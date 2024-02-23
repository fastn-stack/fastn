
-- fastn_mail_request
-- mail entry worker will add mail entry in this table
-- mail dispatch worker will do the actual mail sending process
CREATE TABLE IF NOT EXISTS fastn_mail_request (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT REFERENCES fastn_user(id) ON DELETE CASCADE NOT NULL,
    email email NOT NULL,
    ekind TEXT NOT NULL,
    priority TEXT NOT NULL, -- make this to enum
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    sent_at TIMESTAMP WITH TIME ZONE NULL
);
