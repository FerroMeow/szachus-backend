use std::env;

use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    exp: usize,
    iss: String, // Optional. Issuer
    sub: String, // Optional. Subject (whom token refers to)
}

pub fn create_token(username: &str) -> anyhow::Result<String> {
    let now = chrono::Local::now();
    let expiration_date = now + chrono::TimeDelta::hours(24);
    let claims: Claims = Claims {
        exp: expiration_date.timestamp() as usize,
        iss: "szachus-game".into(),
        sub: username.into(),
    };
    let jwt_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_base64_secret(&env::var("JWT_SECRET")?)?,
    )?;
    Ok(jwt_token)
}
