use async_graphql::{Context, Error, Object};
use sqlx::{Pool, Row, Sqlite};

use crate::{
    authentication::jwt::{authorize, create_jwt},
    db_config::user::check_user,
};

use super::mutations::TokenInfo;

#[derive(sqlx::FromRow, async_graphql::SimpleObject)]
pub struct UserData {
    pub title: String,
    pub url: String,
}

#[derive(sqlx::FromRow, async_graphql::SimpleObject)]
pub struct UserInfo {
    pub name: String,
    pub email: String,
}

#[derive(sqlx::FromRow, async_graphql::SimpleObject)]
pub struct TitleData {
    pub title: String,
    pub data: String,
    pub edit_action: bool,
}
pub struct Query;
#[Object]
impl Query {
    pub async fn testing(&self) -> String {
        "Hello".to_string()
    }

    pub async fn get_user_info(&self, ctx: &Context<'_>) -> async_graphql::Result<UserInfo> {
        let db = ctx.data::<Pool<Sqlite>>().unwrap();
        let uid = ctx.data::<Option<String>>().unwrap();

        let get_user_qry = "SELECT name,email from USERS where id = $1;";

        match sqlx::query(&get_user_qry).bind(&uid).fetch_one(db).await {
            Ok(v) => {
                let name: String = v.get("name");
                let email: String = v.get("email");
                Ok(UserInfo { name, email })
            }
            Err(e) => {
                println!("Error get user info = {:?}", e);
                Err(Error::new("Internal Server Error"))
            }
        }
    }

    pub async fn get_title_list(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<UserData>> {
        let db = ctx.data::<Pool<Sqlite>>().unwrap();
        let uid = ctx.data::<Option<String>>().unwrap();

        let get_title_url = "SELECT a.user_id,c.title,d.url FROM user_data a,data_url b,copied_data c,url d where a.user_id = $1 and a.data_id = c.id and b.data_id = a.data_id and b.url_id = d.id;";

        match sqlx::query(&get_title_url).bind(&uid).fetch_all(db).await {
            Ok(v) => {
                let mut res: Vec<UserData> = Vec::new();
                for i in v.iter() {
                    let title: String = i.get("title");
                    let url: String = i.get("url");
                    res.push(UserData { title, url });
                }
                Ok(res)
            }
            Err(e) => {
                println!("Error get title list = {:?}", e);
                return Err(Error::new("Internal Server Error"));
            }
        }
    }

    pub async fn get_title_data(
        &self,
        ctx: &Context<'_>,
        url_token: String,
    ) -> async_graphql::Result<TitleData> {
        let db = ctx.data::<Pool<Sqlite>>().unwrap();
        let uid = ctx.data::<Option<String>>().unwrap();
        let uid = match uid {
            Some(v) => v.to_string(),
            None => "0".to_string(),
        };
        let get_data_qry = "SELECT a.data,a.title,d.user_id FROM copied_data a , data_url b,url c, user_data d where c.url = $1 and c.id = b.url_id and b.data_id = a.id and a.id = d.data_id;";

        match sqlx::query(&get_data_qry)
            .bind(&url_token)
            .fetch_one(db)
            .await
        {
            Ok(v) => {
                let data: String = v.get("data");
                let title: String = v.get("title");
                let db_uid: i32 = v.get("user_id");
                let mut action = true;
                if db_uid != uid.parse::<i32>().unwrap() {
                    action = false;
                }
                Ok(TitleData {
                    title,
                    data,
                    edit_action: action,
                })
            }
            Err(e) => {
                println!("Error get title data = {:?}", e);
                match e {
                    sqlx::Error::RowNotFound => Err(Error::new("Data Not Found")),
                    _ => Err(Error::new("Internal Server Error")),
                }
            }
        }
    }
}
