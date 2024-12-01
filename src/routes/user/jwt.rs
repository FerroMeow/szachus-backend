use std::env;

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub exp: usize,
    pub iss: String, // Optional. Issuer
    pub sub: i32,    // Optional. Subject (whom token refers to)
}

impl TryFrom<String> for Claims {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        parse_token(&value)
    }
}

pub fn create_token(user_id: i32) -> anyhow::Result<String> {
    let now = chrono::Local::now();
    let expiration_date = now + chrono::TimeDelta::hours(24);
    let claims: Claims = Claims {
        exp: expiration_date.timestamp() as usize,
        iss: "szachus-game".into(),
        sub: user_id,
    };
    let jwt_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_base64_secret(&env::var("JWT_SECRET")?)?,
    )?;
    Ok(jwt_token)
}

pub fn parse_token(token_str: &str) -> anyhow::Result<Claims> {
    decode::<Claims>(
        token_str,
        &DecodingKey::from_base64_secret(&env::var("JWT_SECRET")?)?,
        &Validation::new(jsonwebtoken::Algorithm::HS256),
    )
    .map_or_else(|error| anyhow::bail!(error), |token| Ok(token.claims))
}
