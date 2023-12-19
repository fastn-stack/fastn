-- This file should undo anything in `up.sql`

ALTER TABLE fastn_user ALTER COLUMN username DROP NOT NULL;
ALTER TABLE fastn_user ALTER COLUMN password DROP NOT NULL;
ALTER TABLE fastn_user ALTER COLUMN created_at DROP NOT NULL;
ALTER TABLE fastn_user ALTER COLUMN updated_at DROP NOT NULL;

ALTER TABLE fastn_session ALTER COLUMN user_id DROP NOT NULL;
ALTER TABLE fastn_session ALTER COLUMN created_at DROP NOT NULL;
ALTER TABLE fastn_session ALTER COLUMN updated_at DROP NOT NULL;

ALTER TABLE fastn_oauthtoken ALTER COLUMN session_id DROP NOT NULL;
ALTER TABLE fastn_oauthtoken ALTER COLUMN created_at DROP NOT NULL;
ALTER TABLE fastn_oauthtoken ALTER COLUMN updated_at DROP NOT NULL;

ALTER TABLE fastn_user_email ALTER COLUMN user_id DROP NOT NULL;
ALTER TABLE fastn_user_email ALTER COLUMN verified DROP NOT NULL;
ALTER TABLE fastn_user_email ALTER COLUMN "primary" DROP NOT NULL;
ALTER TABLE fastn_user_email ALTER COLUMN created_at DROP NOT NULL;

ALTER TABLE fastn_email_confirmation ALTER COLUMN email_id DROP NOT NULL;
ALTER TABLE fastn_email_confirmation ALTER COLUMN created_at DROP NOT NULL;
ALTER TABLE fastn_email_confirmation ALTER COLUMN sent_at DROP NOT NULL;
ALTER TABLE fastn_email_confirmation ALTER COLUMN "key" DROP NOT NULL;

