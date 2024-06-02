use std::env;

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    exp: usize,
    iss: String, // Optional. Issuer
    sub: String, // Optional. Subject (whom token refers to)
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;
        parse_token(bearer.token()).map_err(|_| StatusCode::UNAUTHORIZED)
    }
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

pub fn parse_token(token_str: &str) -> anyhow::Result<Claims> {
    decode::<Claims>(
        token_str,
        &DecodingKey::from_base64_secret(&env::var("JWT_SECRET")?)?,
        &Validation::new(jsonwebtoken::Algorithm::HS256),
    )
    .map_or_else(|error| anyhow::bail!(error), |token| Ok(token.claims))
}
