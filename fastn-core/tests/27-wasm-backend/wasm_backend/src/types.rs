use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SubscribeData {
    package: String,
    email: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct LikeData {
    package: String,
    post: String,
}
