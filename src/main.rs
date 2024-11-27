use std::{future::IntoFuture, sync::Mutex};

use actix_cors::Cors;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse},
    guard,
    http::{self, StatusCode},
    web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_actix_web::GraphQL;
use authentication::auth::{login, sign_up};
use db_config::init_db::db_init;
use futures_util::FutureExt;
use graphql_config::{init_graphql::graphql_handler, mutations::Mutation, queries::Query};
use serde_json::json;
use sqlx::{Pool, Sqlite};

pub mod db_config {
    pub mod copied_data;
    pub mod init_db;
    pub mod user;
}
pub mod authentication {
    pub mod auth;
    pub mod jwt;
}
pub mod graphql_config {
    pub mod init_graphql;
    pub mod mutations;
    pub mod queries;
}

pub async fn test() -> impl Responder {
    format!("Hello world")
}

async fn auth_middleware(req: ServiceRequest) -> Result<ServiceRequest, Error> {
    println!("req = {:?}", req);
    Ok(req)
}

type MySchema = Schema<Query, Mutation, EmptySubscription>;
pub struct AppState {
    pub schema: Mutex<MySchema>,
    pub db: Mutex<Pool<Sqlite>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, world!");
    let db = match db_init().await {
        Ok(v) => v,
        Err(e) => {
            panic!("{:?}", e);
        }
    };
    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(db.clone())
        .finish();
    let appstate = web::Data::new(AppState {
        schema: Mutex::new(schema),
        db: Mutex::new(db.clone()),
    });
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::CONTENT_TYPE,
            ])
            .max_age(3600);
        App::new()
            .wrap(cors)
            .app_data(appstate.clone())
            // .service(
            //     web::resource("/graphql")
            //         .to(graphql_handler),
            // )
            .route("/graphql", web::post().to(graphql_handler))
            .route("/login", web::post().to(login))
            .route("/signup", web::post().to(sign_up))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
