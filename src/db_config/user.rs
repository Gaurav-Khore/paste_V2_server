use async_graphql::Error;
use sqlx::{Pool, Row, Sqlite};

pub async fn insert_user(
    db: &Pool<Sqlite>,
    name: String,
    password: String,
    email: String,
) -> Result<i32, Error> {
    let check_qry = "Select EXISTS (SELECT 1 FROM USERS where name like $1);";

    let check = match sqlx::query(&check_qry).bind(&name).fetch_one(db).await {
        Ok(v) => v,
        Err(e) => {
            return Err(e.into());
        }
    };
    // println!("checl = {:?}",check);
    let check: bool = check.get(0);
    if check {
        return Err("User Already present".into());
    }

    let insert_qry = "INSERT INTO users(name,email,password) values ($1,$2,$3);";
    let get_id_qry = "Select ID FROM USERS where email like $1;";
    match sqlx::query(&insert_qry)
        .bind(&name)
        .bind(&email)
        .bind(&password)
        .execute(db)
        .await
    {
        Ok(_) => match sqlx::query(&get_id_qry).bind(&email).fetch_one(db).await {
            Ok(v) => {
                let id: i32 = v.get("id");
                return Ok(id);
            }
            Err(e) => {
                return Err(Error::new("Internal Server Error"));
            }
        },
        Err(e) => {
            println!("Error insert users = {:?}", e);
            return Err(Error::new("Unable to SignUp"));
        }
    }
}

pub async fn check_user(db: &Pool<Sqlite>, email: String, password: String) -> Result<i32, Error> {
    let get_user_qry = "SELECT * from users where email = $1;";
    match sqlx::query(&get_user_qry).bind(&email).fetch_one(db).await {
        Ok(v) => {
            let db_email: String = v.get("email");
            let db_passwd: String = v.get("password");
            let uid: i32 = v.get("id");
            if db_email != email || db_passwd != password {
                return Err(Error::new("Invalid Credentials"));
            }
            return Ok(uid);
        }
        Err(e) => {
            println!("Error check user = {:?}", e);
            match e {
                sqlx::Error::RowNotFound => {
                    return Err(Error::new("Invalid Credentials"));
                }
                _ => {
                    return Err(Error::new("Internal Server Error"));
                }
            }
        }
    }
}
