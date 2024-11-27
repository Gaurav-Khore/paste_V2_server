use std::sync::Mutex;

use actix_web::{HttpMessage, HttpRequest};
use async_graphql::Error;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};

const JWT_SECRET: &[u8] = b"pastebin";

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn create_jwt(uid: &String) -> Result<String, jsonwebtoken::errors::Error> {
    println!("Hello from the create jwt token");
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::minutes(15))
        .expect("Failed to create the expiration")
        .timestamp();

    let claim = Claims {
        sub: uid.to_string(),
        exp: expiration as usize,
    };

    let header = Header::new(jsonwebtoken::Algorithm::HS256);
    encode(&header, &claim, &EncodingKey::from_secret(JWT_SECRET))
}

pub fn extract_jwt(req: Mutex<HttpRequest>) -> Option<String> {
    if let Some(auth_header) = req.lock().unwrap().headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer") {
                let token = auth_str.trim_start_matches("Bearer ");
                return Some(token.to_string());
            }
        }
    }
    None
}

pub fn decode_jwt(token: Option<String>) -> async_graphql::Result<Claims> {
    if token.is_none() {
        return Err(Error::new("Not Authorized"));
    }
    let token = token.unwrap();
    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    match decode::<Claims>(&token, &DecodingKey::from_secret(JWT_SECRET), &validation) {
        Ok(token) => Ok(token.claims),
        Err(e) => Err(Error::new("Invalid Authorization")),
    }
}

pub async fn authorize(db: &Pool<Sqlite>, token: Option<String>) -> async_graphql::Result<String> {
    match decode_jwt(token) {
        Ok(claim) => {
            let uid = claim.sub;
            let check_qry = "SELECT EXISTS (SELECT 1 FROM USERS WHERE id = $1);";
            match sqlx::query(&check_qry).bind(&uid).fetch_one(db).await {
                Ok(v) => {
                    let check: bool = v.get(0);
                    if !check {
                        return Err(Error::new("Internal Server Error"));
                    }
                    Ok(uid)
                }
                Err(e) => match e {
                    sqlx::Error::RowNotFound => Err(Error::new("User Not Found")),
                    _ => Err(Error::new("Internal Server Error")),
                },
            }
        }
        Err(e) => Err(e),
    }
}
