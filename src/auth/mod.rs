pub(crate) mod discord;
pub(crate) mod github;
pub(crate) mod gmail;
pub(crate) mod slack;
pub(crate) mod telegram;

pub(crate) use github::auth;
pub(crate) use github::get_identity;
pub(crate) use github::index;
pub(crate) use github::login;
pub(crate) use github::logout;
