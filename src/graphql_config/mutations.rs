use async_graphql::{Context, Error, Object};
use sqlx::{Pool, Sqlite};

use crate::{
    authentication::jwt::{authorize, create_jwt, decode_jwt},
    db_config::{copied_data::insert_data, user::insert_user},
};

#[derive(sqlx::FromRow, async_graphql::SimpleObject)]
pub struct TokenInfo {
    pub token: String,
}

#[derive(sqlx::FromRow, async_graphql::SimpleObject)]
pub struct UrlInfo {
    pub url: String,
}
pub struct Mutation;
#[Object]
impl Mutation {
    pub async fn insert_data(
        &self,
        ctx: &Context<'_>,
        data: String,
        title: String,
    ) -> async_graphql::Result<UrlInfo> {
        let uid = ctx.data::<Option<String>>().unwrap();
        let db = ctx.data::<Pool<Sqlite>>().unwrap();
        println!("data = {:?}", data);
        match insert_data(db, uid.clone().unwrap(), data, title).await {
            Ok(v) => Ok(UrlInfo { url: v }),
            Err(e) => Err(e.into()),
        }
    }
}
