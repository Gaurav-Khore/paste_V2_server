use async_graphql::Error;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sqlx::{Pool, Row, Sqlite};

pub async fn insert_data(
    db: &Pool<Sqlite>,
    id: String,
    text: String,
    title: String,
) -> async_graphql::Result<String> {
    println!("Hello from the insert data");
    let check_data = "SELECT EXISTS (SELECT 1 from user_data a, copied_data b where a.user_id = $1 and a.data_id = b.id and b.data like $2 and b.title = $3);";
    let insert_tex_qry = "INSERT INTO copied_data (data,title) VALUES($1,$2);";
    let insert_url = "INSERT INTO url (url) VALUES ($1);";
    let insert_data_url = "INSERT INTO data_url(data_id,url_id) VALUES((select id from copied_data where data like $1 and title like $2 and id not in (SELECT data_id from user_data)),(select id from url where url like $3));";
    let insert_user_url =
        "INSERT INTO user_url(user_id,url_id) VALUES ($1,(select id from url where url like $2));";
    let insert_user_data = "INSERT INTO user_data(user_id,data_id) VALUES ($1,(select id from copied_data where data like $2 and title like $3 and id not in (SELECT data_id from user_data)));";
    match sqlx::query(&check_data)
        .bind(&id)
        .bind(&text)
        .bind(&title)
        .fetch_one(db)
        .await
    {
        Ok(v) => {
            let check: bool = v.get(0);
            if check {
                return Err(Error::new("Data Already Present"));
            }
            match sqlx::query(&insert_tex_qry)
                .bind(&text)
                .bind(&title)
                .execute(db)
                .await
            {
                Ok(_) => {
                    let url: String = thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(10)
                        .map(char::from)
                        .collect();
                    match sqlx::query(&insert_url).bind(&url).execute(db).await {
                        Ok(_) => {
                            match sqlx::query(&insert_data_url)
                                .bind(&text)
                                .bind(&title)
                                .bind(&url)
                                .execute(db)
                                .await
                            {
                                Ok(_) => {
                                    match sqlx::query(&insert_user_url)
                                        .bind(&id)
                                        .bind(&url)
                                        .execute(db)
                                        .await
                                    {
                                        Ok(_) => {
                                            match sqlx::query(&insert_user_data)
                                                .bind(&id)
                                                .bind(&text)
                                                .bind(&title)
                                                .execute(db)
                                                .await
                                            {
                                                Ok(_) => Ok(url),
                                                Err(e) => {
                                                    println!("user data = {:?}", e);
                                                    return Err(Error::new(
                                                        "Internal Server Error",
                                                    ));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            println!("user url = {:?}", e);
                                            return Err(Error::new("Internal Server Error"));
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("data url = {:?}", e);
                                    return Err(Error::new("Internal Server Error"));
                                }
                            }
                        }
                        Err(e) => {
                            println!("insert url = {:?}", e);
                            return Err(Error::new("Internal Server Error"));
                        }
                    }
                }
                Err(e) => Err(e.into()),
            }
        }
        Err(e) => Err(e.into()),
    }
}
