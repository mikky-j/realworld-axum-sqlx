use crate::errors::RequestError;
use anyhow::{Context, Result};
use argon2::PasswordVerifier;
use argon2::{password_hash::SaltString, Argon2, PasswordHash};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

const JWT_EXPIRY_DURATION: time::Duration = time::Duration::days(90);

#[derive(Debug, Serialize, Deserialize)]
struct AuthClaim {
    id: i64,
    exp: i64,
}

pub struct AuthUser {
    pub id: i64,
    pub token: String,
}

pub struct MaybeUser(pub Option<AuthUser>);

impl MaybeUser {
    pub fn get_id(&self) -> Option<i64> {
        self.0.as_ref().map(|a| a.id)
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for MaybeUser
where
    S: Send + Sync + 'static,
{
    type Rejection = RequestError;
    async fn from_request_parts(
        parts: &mut Parts,
        _: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let header = match parts.headers.get("Authorization") {
            Some(header) => header,
            None => return Ok(MaybeUser(None)),
        };
        let header = match header.to_str() {
            Ok(header) => header,
            Err(_) => {
                println!("Error converting header to str");
                return Err(RequestError::NotAuthorized("Invalid token"));
            }
        };

        let token = match header.strip_prefix("Token ") {
            Some(token) => token,
            None => {
                println!("Error stripping prefix");
                return Err(RequestError::NotAuthorized("Invalid token"));
            }
        };

        let id = match verify_jwt_token(token) {
            Ok(id) => id,
            Err(e) => return Err(e),
        };

        Ok(MaybeUser(Some(AuthUser {
            id,
            token: token.to_string(),
        })))
    }
}

pub fn get_jwt_token(id: i64) -> Result<String> {
    let jwt_secret = std::env::var("JWT_SECRET").context("Failed to get JWT_SECRET")?;
    let expiry_date = OffsetDateTime::now_utc() + JWT_EXPIRY_DURATION;
    let claim = AuthClaim {
        id,
        exp: expiry_date.unix_timestamp(),
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claim,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .context("Failed to generate jwt token");
    token
}

pub fn verify_jwt_token(token: &str) -> Result<i64, RequestError> {
    let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| RequestError::ServerError)?;
    let token_data = jsonwebtoken::decode::<AuthClaim>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_ref()),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|e| {
        println!("Error verifying token:\n {}", e);
        RequestError::NotAuthorized("Invalid Token")
    })?;
    let claim = token_data.claims;
    if claim.exp < OffsetDateTime::now_utc().unix_timestamp() {
        return Err(RequestError::NotAuthorized("Token expired"));
    }
    Ok(claim.id)
}

pub async fn verify_password_argon2(password: String, hash: &str) -> Result<bool> {
    let hash = hash.to_owned();
    tokio::task::spawn_blocking(move || {
        let hash = PasswordHash::new(hash.as_str())
            .map_err(|_| anyhow::anyhow!("Failed to verify password"))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .map_err(|_| anyhow::anyhow!("Failed to verify password"))
            .is_ok())
    })
    .await
    .context("Failed to verify password")?
}

pub async fn hash_password_argon2(password: String) -> Result<String> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(rand::thread_rng());
        let hash = PasswordHash::generate(Argon2::default(), password, salt.as_salt())
            .map_err(|_| anyhow::anyhow!("Failed to hash password"))?;
        Ok(hash.to_string())
    })
    .await
    .context("Failed to hash password")?
}
