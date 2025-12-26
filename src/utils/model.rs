use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequests {
    pub username: String,
    pub password: String,
}
