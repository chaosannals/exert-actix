use actix_web::{get, Responder};
use sqlx::{Connection, Executor, SqliteConnection};

async fn d() -> Result<(), sqlx::Error> {
    let mut conn = SqliteConnection::connect("sqlite:exert.db").await?;
    conn.execute(
        "CREATE TABLE e_test (
        id INTEGER PRIMARY KEY ASC AUTOINCREMENT,
        name VARCHAR(50) 
    )",
    )
    .await?;
    Ok(())
}

#[get("/d.html")]
async fn do_d() -> impl Responder {
    match d().await {
        Ok(i) => format!("Ok {:?}", i),
        Err(e) => format!("Error {:?}", e),
    }
}
