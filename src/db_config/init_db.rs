use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite, SqlitePool,
};

pub async fn db_init() -> Result<Pool<Sqlite>, sqlx::Error> {
    println!("Hello from the db_init");
    let option = SqliteConnectOptions::new()
        .filename("pastebin.db")
        .create_if_missing(true);
    let db = match SqlitePool::connect_with(option).await {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    println!("Successfully connected to db");
    let db_clone = db.clone();
    init_table(&db_clone).await;
    Ok(db)
}

async fn init_table(db: &Pool<Sqlite>) {
    //create user table
    sqlx::query("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY AUTOINCREMENT,name TEXT not null, email TEXT UNIQUE NOT NULL,password TEXT);").execute(db).await.expect("Failed to create the user table");

    // create url table
    sqlx::query("CREATE TABLE IF NOT EXISTS url (id INTEGER PRIMARY KEY AUTOINCREMENT,url TEXT);")
        .execute(db)
        .await
        .expect("Failed to create Url table");

    //create copied_Data table
    sqlx::query("CREATE TABLE IF NOT EXISTS copied_data (id INTEGER PRIMARY KEY AUTOINCREMENT,data TEXT,title TEXT);")
        .execute(db)
        .await
        .expect("Failed to create coiped data table");

    // create user_url table
    sqlx::query("CREATE TABLE IF NOT EXISTS user_url (user_id INTEGER REFERENCES users(id),url_id INTEGER REFERENCES url(id));").execute(db).await.expect("Failed to create user_url table");

    //create data_url table
    sqlx::query("CREATE TABLE IF NOT EXISTS data_url (data_id INTEGER REFERENCES copied_data(id) , url_id INTEGER REFERENCES url(id) );").execute(db).await.expect("Failed to create data url table");

    // create user_data table
    sqlx::query("CREATE TABLE IF NOT EXISTS user_data(user_id INTEGER REFERENCES users(id),data_id INTEGER REFERENCES copied_data(id));").execute(db).await.expect("Failed to create the user_data table");
}
