use actix_web::{http::StatusCode, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Pool, Sqlite};

use crate::{
    authentication::jwt::create_jwt,
    db_config::user::{check_user, insert_user},
    graphql_config::mutations::TokenInfo,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct UserCreds {
    email: String,
    password: String,
}

pub async fn login(creds: web::Json<UserCreds>, data: web::Data<AppState>) -> impl Responder {
    println!("Hello from the login");
    println!("data = {:?}", creds.email.clone());
    let db = data.db.lock().unwrap();
    match check_user(&db, creds.email.clone(), creds.password.clone()).await {
        Ok(v) => match create_jwt(&v.to_string()) {
            Ok(v) => {
                return HttpResponse::Ok().json(json!({
                    "token": v
                }));
            }
            Err(e) => {
                return HttpResponse::Forbidden().json(json!( {
                    "Error": "Wrong Credentials"
                }));
            }
        },
        Err(e) => {
            return HttpResponse::Forbidden().json(json!( {
                "Error": "Wrong Credentials"
            }));
        }
    }
}

#[derive(Deserialize)]
pub struct NewCreds {
    name: String,
    email: String,
    password: String,
}
pub async fn sign_up(newcreds: web::Json<NewCreds>, data: web::Data<AppState>) -> impl Responder {
    let db = data.db.lock().unwrap();

    let id = match insert_user(
        &db,
        newcreds.name.clone(),
        newcreds.password.clone(),
        newcreds.email.clone(),
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            println!("Error signUp = {:?}", e);
            return HttpResponse::Forbidden().json(json!( {
                "Error": "Wrong Credentials"
            }));
        }
    };

    match create_jwt(&format!("{:?}", id)) {
        Ok(v) => {
            return HttpResponse::Ok().json(json!({
                "token": v
            }));
        }
        Err(e) => {
            println!("Error signUp = {:?}", e);
            return HttpResponse::Forbidden().json(json!( {
                "Error": "Wrong Credentials"
            }));
        }
    }
}
