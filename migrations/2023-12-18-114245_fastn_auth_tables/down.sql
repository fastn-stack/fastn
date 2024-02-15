-- This file should undo anything in `up.sql`

DROP TABLE IF EXISTS fastn_email_confirmation;
DROP TABLE IF EXISTS fastn_user_email;
DROP TABLE IF EXISTS fastn_oauthtoken;
DROP TABLE IF EXISTS fastn_auth_session;
DROP TABLE IF EXISTS fastn_user;

DROP DOMAIN IF EXISTS email;
DROP EXTENSION IF EXISTS citext;

