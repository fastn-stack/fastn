// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "citext"))]
    pub struct Citext;
}

diesel::table! {
    fastn_auth_session (id) {
        id -> Int8,
        user_id -> Int8,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    fastn_email_confirmation (id) {
        id -> Int8,
        email_id -> Int8,
        session_id -> Int8,
        created_at -> Timestamptz,
        sent_at -> Timestamptz,
        key -> Text,
    }
}

diesel::table! {
    fastn_oauthtoken (id) {
        id -> Int8,
        session_id -> Int8,
        token -> Text,
        provider -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    fastn_password_reset (id) {
        id -> Int8,
        user_id -> Int8,
        created_at -> Timestamptz,
        sent_at -> Timestamptz,
        key -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Citext;

    fastn_user (id) {
        id -> Int8,
        username -> Text,
        password -> Text,
        email -> Citext,
        verified_email -> Bool,
        name -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Citext;

    fastn_user_email (id) {
        id -> Int8,
        user_id -> Int8,
        email -> Citext,
        verified -> Bool,
        primary -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(fastn_auth_session -> fastn_user (user_id));
diesel::joinable!(fastn_email_confirmation -> fastn_auth_session (session_id));
diesel::joinable!(fastn_email_confirmation -> fastn_user_email (email_id));
diesel::joinable!(fastn_oauthtoken -> fastn_auth_session (session_id));
diesel::joinable!(fastn_password_reset -> fastn_user (user_id));
diesel::joinable!(fastn_user_email -> fastn_user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    fastn_auth_session,
    fastn_email_confirmation,
    fastn_oauthtoken,
    fastn_password_reset,
    fastn_user,
    fastn_user_email,
);
