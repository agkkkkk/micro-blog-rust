use base64::{engine::general_purpose, Engine as _};
use chrono::Duration;
use jsonwebtoken::errors::Error;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub access_token: Option<String>,
    pub user: String,
    pub token_uuid: Uuid,
    pub expires_in: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub token_uuid: String,
    pub user: String,
    pub exp: i64,
}

pub fn generate_jwt_token(user_id: String, ttl: i64, private_key: String) -> Result<Token, Error> {
    let bytes_private_key = general_purpose::STANDARD.decode(private_key).unwrap();
    let decoded_private_key = String::from_utf8(bytes_private_key).unwrap();

    let now = chrono::Utc::now();

    let mut token = Token {
        access_token: None,
        user: user_id,
        token_uuid: Uuid::new_v4(),
        expires_in: Some((now + Duration::minutes(ttl)).timestamp()),
    };

    let token_claims = TokenClaims {
        token_uuid: token.token_uuid.to_string(),
        user: token.user.to_string(),
        exp: token.expires_in.unwrap(),
    };

    let header = Header::new(Algorithm::RS256);

    let access_token = jsonwebtoken::encode(
        &header,
        &token_claims,
        &EncodingKey::from_rsa_pem(decoded_private_key.as_bytes())?,
    )?;

    token.access_token = Some(access_token);

    Ok(token)
}

pub fn verify_jwt_token(token: &str, public_key: String) -> Result<Token, Error> {
    let bytes_public_key = general_purpose::STANDARD.decode(public_key).unwrap();
    let decoded_public_key = String::from_utf8(bytes_public_key).unwrap();

    let validation_algo = Validation::new(Algorithm::RS256);

    let decoded_token = jsonwebtoken::decode::<TokenClaims>(
        token,
        &DecodingKey::from_rsa_pem(decoded_public_key.as_bytes())?,
        &validation_algo,
    )?;

    let user = decoded_token.claims.user;
    let token_uuid = Uuid::from_str(decoded_token.claims.token_uuid.as_str()).unwrap();

    Ok(Token {
        access_token: None,
        user: user,
        token_uuid: token_uuid,
        expires_in: None,
    })
}
