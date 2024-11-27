use std::sync::Mutex;

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use async_graphql::{Error, Pos, Response, ServerError};
use async_graphql_actix_web::{GraphQL, GraphQLRequest, GraphQLResponse};

use crate::{
    authentication::jwt::{authorize, extract_jwt},
    AppState,
};

pub async fn graphql_handler(
    data: web::Data<AppState>,
    gql_request: GraphQLRequest,
    req: HttpRequest,
) -> GraphQLResponse {
    println!("Hello from the qraphql Handler");
    let schema = data.schema.lock().unwrap();
    // handle req and pass the token
    let token = extract_jwt(Mutex::new(req));
    let mut uid = None;
    if token.is_none() || token == Some("null".to_string()) {
        let res = Response::from_errors(vec![ServerError::new("Invalid Credentials", None)]);
        if !gql_request.0.query.contains("GetTitleData") {
                return Response::from_errors(vec![ServerError::new("Not Authorizerd", None)])
                    .into();
        
        }
    } else {
        let pool = data.db.lock().unwrap();
        uid = match authorize(&pool, token.clone()).await {
            Ok(v) => Some(v),
            Err(e) => {
                return Response::from_errors(vec![ServerError::new(e.message, None)]).into();
            }
        };
    }

    let ctx = gql_request.into_inner().data(uid);
    schema.execute(ctx).await.into()
}
