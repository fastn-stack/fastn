-- Your SQL goes here

ALTER TABLE fastn_user ALTER COLUMN username SET NOT NULL;
ALTER TABLE fastn_user ALTER COLUMN password SET NOT NULL;
ALTER TABLE fastn_user ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE fastn_user ALTER COLUMN updated_at SET NOT NULL;

ALTER TABLE fastn_session ALTER COLUMN user_id SET NOT NULL;
ALTER TABLE fastn_session ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE fastn_session ALTER COLUMN updated_at SET NOT NULL;

ALTER TABLE fastn_oauthtoken ALTER COLUMN session_id SET NOT NULL;
ALTER TABLE fastn_oauthtoken ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE fastn_oauthtoken ALTER COLUMN updated_at SET NOT NULL;

ALTER TABLE fastn_user_email ALTER COLUMN user_id SET NOT NULL;
ALTER TABLE fastn_user_email ALTER COLUMN verified SET NOT NULL;
ALTER TABLE fastn_user_email ALTER COLUMN "primary" SET NOT NULL;
ALTER TABLE fastn_user_email ALTER COLUMN created_at SET NOT NULL;

ALTER TABLE fastn_email_confirmation ALTER COLUMN email_id SET NOT NULL;
ALTER TABLE fastn_email_confirmation ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE fastn_email_confirmation ALTER COLUMN sent_at SET NOT NULL;
ALTER TABLE fastn_email_confirmation ALTER COLUMN "key" SET NOT NULL;

