use actix_web::dev::Payload;
use actix_web::error::ErrorUnauthorized;
use actix_web::{http, Error as ActixWebError, FromRequest, HttpRequest};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::env;
use std::future::{ready, Ready};

use crate::token;

#[derive(Debug, Serialize, Deserialize)]
pub struct JWTAuthToken {
    user: String,
    access_token: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

impl FromRequest for JWTAuthToken {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let access_token_public_key =
            env::var("BASE64_ACCESS_TOKEN_PUBLIC_KEY").expect("Failed to fetch access token ");

        let access_token = req
            .cookie("access_token")
            .map(|cookie| cookie.value().to_string())
            .or_else(|| {
                req.headers()
                    .get(http::header::AUTHORIZATION)
                    .map(|header| header.to_str().unwrap().split_at(7).1.to_string())
                // split at 7 is to separate Bearer keyword and token.
            });

        println!("{:?}", access_token);

        if access_token.is_none() {
            let json_error = ErrorResponse {
                status: "FAILED".to_string(),
                message: "You are not logged in, please provide token".to_string(),
            };
            return ready(Err(ErrorUnauthorized(json_error)));
        };

        let access_token_details =
            match token::verify_jwt_token(&access_token.unwrap(), access_token_public_key) {
                Ok(res) => res,
                Err(err) => {
                    let json_error = ErrorResponse {
                        status: "FAILED".to_string(),
                        message: format!("{:?}", err),
                    };
                    return ready(Err(ErrorUnauthorized(json_error)));
                }
            };

        let user = access_token_details.user;
        let access_jwt_token = access_token_details.access_token.unwrap();

        ready(Ok(JWTAuthToken {
            user: user,
            access_token: access_jwt_token,
        }))
    }
}
