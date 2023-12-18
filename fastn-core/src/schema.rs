// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "citext"))]
    pub struct Citext;
}

diesel::table! {
    fastn_email_confirmation (id) {
        id -> Int4,
        email_id -> Nullable<Int4>,
        created_at -> Nullable<Timestamptz>,
        sent_at -> Nullable<Timestamptz>,
        key -> Nullable<Text>,
    }
}

diesel::table! {
    fastn_oauthtoken (id) {
        id -> Int4,
        session_id -> Nullable<Int4>,
        token -> Text,
        provider -> Text,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    fastn_session (id) {
        id -> Int4,
        user_id -> Nullable<Int4>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    fastn_user (id) {
        id -> Int4,
        username -> Nullable<Text>,
        password -> Nullable<Text>,
        name -> Text,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Citext;

    fastn_user_email (id) {
        id -> Int4,
        user_id -> Nullable<Int4>,
        email -> Citext,
        verified -> Nullable<Bool>,
        primary -> Nullable<Bool>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(fastn_email_confirmation -> fastn_user_email (email_id));
diesel::joinable!(fastn_oauthtoken -> fastn_session (session_id));
diesel::joinable!(fastn_session -> fastn_user (user_id));
diesel::joinable!(fastn_user_email -> fastn_user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    fastn_email_confirmation,
    fastn_oauthtoken,
    fastn_session,
    fastn_user,
    fastn_user_email,
);
