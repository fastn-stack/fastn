// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "citext"))]
    pub struct Citext;
}

diesel::table! {
    auth_group (id) {
        id -> Int4,
        #[max_length = 150]
        name -> Varchar,
    }
}

diesel::table! {
    auth_group_permissions (id) {
        id -> Int8,
        group_id -> Int4,
        permission_id -> Int4,
    }
}

diesel::table! {
    auth_permission (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        content_type_id -> Int4,
        #[max_length = 100]
        codename -> Varchar,
    }
}

diesel::table! {
    auth_user (id) {
        id -> Int4,
        #[max_length = 128]
        password -> Varchar,
        last_login -> Nullable<Timestamptz>,
        is_superuser -> Bool,
        #[max_length = 150]
        username -> Varchar,
        #[max_length = 150]
        first_name -> Varchar,
        #[max_length = 150]
        last_name -> Varchar,
        #[max_length = 254]
        email -> Varchar,
        is_staff -> Bool,
        is_active -> Bool,
        date_joined -> Timestamptz,
    }
}

diesel::table! {
    auth_user_groups (id) {
        id -> Int8,
        user_id -> Int4,
        group_id -> Int4,
    }
}

diesel::table! {
    auth_user_user_permissions (id) {
        id -> Int8,
        user_id -> Int4,
        permission_id -> Int4,
    }
}

diesel::table! {
    django_admin_log (id) {
        id -> Int4,
        action_time -> Timestamptz,
        object_id -> Nullable<Text>,
        #[max_length = 200]
        object_repr -> Varchar,
        action_flag -> Int2,
        change_message -> Text,
        content_type_id -> Nullable<Int4>,
        user_id -> Int4,
    }
}

diesel::table! {
    django_content_type (id) {
        id -> Int4,
        #[max_length = 100]
        app_label -> Varchar,
        #[max_length = 100]
        model -> Varchar,
    }
}

diesel::table! {
    django_migrations (id) {
        id -> Int8,
        #[max_length = 255]
        app -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        applied -> Timestamptz,
    }
}

diesel::table! {
    django_session (session_key) {
        #[max_length = 40]
        session_key -> Varchar,
        session_data -> Text,
        expire_date -> Timestamptz,
    }
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

diesel::table! {
    ft_document (id) {
        id -> Int8,
        path -> Text,
        is_public -> Bool,
        is_ftd -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        site_id -> Int8,
        tejar_content_id -> Nullable<Int8>,
    }
}

diesel::table! {
    ft_document_history (id) {
        id -> Int8,
        path -> Text,
        diff -> Nullable<Text>,
        is_deleted -> Bool,
        created_at -> Timestamptz,
        editor_id -> Int8,
        site_id -> Int8,
        tejar_content_id -> Nullable<Int8>,
    }
}

diesel::table! {
    ft_domain (id) {
        id -> Int8,
        domain -> Text,
        dns_status -> Text,
        dns_attempts -> Int4,
        dns_check_scheduled_at -> Timestamptz,
        ssl_status -> Text,
        ssl_http_token -> Nullable<Text>,
        ssl_http_proof -> Nullable<Bytea>,
        ssl_check_scheduled_at -> Nullable<Timestamptz>,
        ssl_certificate_issued_at -> Nullable<Timestamptz>,
        ssl_certificate_pem -> Nullable<Text>,
        ssl_encrypted_private_key_pem -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        site_id -> Int8,
    }
}

diesel::table! {
    ft_environment (id) {
        id -> Int8,
        name -> Text,
        encrypted_value -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        site_id -> Int8,
    }
}

diesel::table! {
    ft_event (id) {
        id -> Int8,
        ip -> Nullable<Inet>,
        user_agent -> Text,
        domain -> Nullable<Text>,
        okind -> Text,
        ekind -> Text,
        outcome -> Text,
        outcome_data -> Jsonb,
        input -> Jsonb,
        count_1 -> Int4,
        response_time_ns -> Int8,
        created_at -> Timestamptz,
        myself -> Nullable<Int8>,
        org_id -> Nullable<Int8>,
        site_id -> Nullable<Int8>,
        someone -> Nullable<Int8>,
        source -> Nullable<Text>,
    }
}

diesel::table! {
    ft_gh_oidc_repo_rule (id) {
        id -> Int8,
        gh_repo -> Text,
        gh_branch -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        site_id -> Int8,
    }
}

diesel::table! {
    ft_notification (id) {
        id -> Int8,
        url -> Text,
        read -> Bool,
        done -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        org_id -> Nullable<Int8>,
        user_id -> Int8,
    }
}

diesel::table! {
    ft_org (id) {
        id -> Int8,
        name -> Text,
        slug -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        owner_id -> Int8,
        plan_id -> Nullable<Int8>,
    }
}

diesel::table! {
    ft_org_member (id) {
        id -> Int8,
        role -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        org_id -> Int8,
        user_id -> Int8,
    }
}

diesel::table! {
    ft_plan (id) {
        id -> Int8,
        name -> Text,
        description -> Text,
    }
}

diesel::table! {
    ft_planhistory (id) {
        id -> Int8,
        created_at -> Timestamptz,
        org_id -> Int8,
        plan_id -> Int8,
    }
}

diesel::table! {
    ft_site (id) {
        id -> Int8,
        name -> Text,
        #[max_length = 50]
        slug -> Varchar,
        is_static -> Bool,
        is_public -> Bool,
        domain -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by -> Int8,
        org_id -> Nullable<Int8>,
        is_editable -> Bool,
    }
}

diesel::table! {
    ft_site_token (id) {
        id -> Int8,
        about -> Text,
        token -> Text,
        can_read -> Bool,
        can_write -> Bool,
        last_used_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by -> Int8,
        site_id -> Int8,
    }
}

diesel::table! {
    ft_tejar_content (id) {
        id -> Int8,
        s3_tejar_file_offset -> Int4,
        s3_tejar_file_size -> Int4,
        sha256_hash -> Text,
        shared_count -> Int4,
        file_id -> Int8,
    }
}

diesel::table! {
    ft_tejar_file (id) {
        id -> Int8,
        shared -> Bool,
        created_at -> Timestamptz,
        upload_session_id -> Nullable<Int8>,
    }
}

diesel::table! {
    ft_upload_session (id) {
        id -> Int8,
        json -> Text,
        bytes_saved -> Int4,
        created_at -> Timestamptz,
        finished_at -> Nullable<Timestamptz>,
        site_id -> Int8,
    }
}

diesel::table! {
    ft_user (id) {
        id -> Int8,
        username -> Text,
        name -> Text,
        #[max_length = 100]
        email -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(auth_group_permissions -> auth_group (group_id));
diesel::joinable!(auth_group_permissions -> auth_permission (permission_id));
diesel::joinable!(auth_permission -> django_content_type (content_type_id));
diesel::joinable!(auth_user_groups -> auth_group (group_id));
diesel::joinable!(auth_user_groups -> auth_user (user_id));
diesel::joinable!(auth_user_user_permissions -> auth_permission (permission_id));
diesel::joinable!(auth_user_user_permissions -> auth_user (user_id));
diesel::joinable!(django_admin_log -> auth_user (user_id));
diesel::joinable!(django_admin_log -> django_content_type (content_type_id));
diesel::joinable!(fastn_auth_session -> fastn_user (user_id));
diesel::joinable!(fastn_email_confirmation -> fastn_auth_session (session_id));
diesel::joinable!(fastn_email_confirmation -> fastn_user_email (email_id));
diesel::joinable!(fastn_oauthtoken -> fastn_auth_session (session_id));
diesel::joinable!(fastn_password_reset -> fastn_user (user_id));
diesel::joinable!(fastn_user_email -> fastn_user (user_id));
diesel::joinable!(ft_document -> ft_site (site_id));
diesel::joinable!(ft_document -> ft_tejar_content (tejar_content_id));
diesel::joinable!(ft_document_history -> ft_site (site_id));
diesel::joinable!(ft_document_history -> ft_tejar_content (tejar_content_id));
diesel::joinable!(ft_document_history -> ft_user (editor_id));
diesel::joinable!(ft_domain -> ft_site (site_id));
diesel::joinable!(ft_environment -> ft_site (site_id));
diesel::joinable!(ft_event -> ft_org (org_id));
diesel::joinable!(ft_event -> ft_site (site_id));
diesel::joinable!(ft_gh_oidc_repo_rule -> ft_site (site_id));
diesel::joinable!(ft_notification -> ft_org (org_id));
diesel::joinable!(ft_notification -> ft_user (user_id));
diesel::joinable!(ft_org -> ft_plan (plan_id));
diesel::joinable!(ft_org -> ft_user (owner_id));
diesel::joinable!(ft_org_member -> ft_org (org_id));
diesel::joinable!(ft_org_member -> ft_user (user_id));
diesel::joinable!(ft_planhistory -> ft_org (org_id));
diesel::joinable!(ft_planhistory -> ft_plan (plan_id));
diesel::joinable!(ft_site -> ft_org (org_id));
diesel::joinable!(ft_site -> ft_user (created_by));
diesel::joinable!(ft_site_token -> ft_site (site_id));
diesel::joinable!(ft_site_token -> ft_user (created_by));
diesel::joinable!(ft_tejar_content -> ft_tejar_file (file_id));
diesel::joinable!(ft_tejar_file -> ft_upload_session (upload_session_id));
diesel::joinable!(ft_upload_session -> ft_site (site_id));

diesel::allow_tables_to_appear_in_same_query!(
    auth_group,
    auth_group_permissions,
    auth_permission,
    auth_user,
    auth_user_groups,
    auth_user_user_permissions,
    django_admin_log,
    django_content_type,
    django_migrations,
    django_session,
    fastn_auth_session,
    fastn_email_confirmation,
    fastn_oauthtoken,
    fastn_password_reset,
    fastn_user,
    fastn_user_email,
    ft_document,
    ft_document_history,
    ft_domain,
    ft_environment,
    ft_event,
    ft_gh_oidc_repo_rule,
    ft_notification,
    ft_org,
    ft_org_member,
    ft_plan,
    ft_planhistory,
    ft_site,
    ft_site_token,
    ft_tejar_content,
    ft_tejar_file,
    ft_upload_session,
    ft_user,
);
