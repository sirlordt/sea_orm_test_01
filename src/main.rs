//use std::error::Error;

/*
You need patch you local sea-orm crate to this work.

src/database/transaction.rs

-- pub(crate) async fn run<F, T, E>(self, callback: F) -> Result<T, TransactionError<E>>
++ pub(crate) async fn run<F, T, E>(mut self, callback: F) -> Result<T, TransactionError<E>>

-- pub async fn commit(mut self) -> Result<(), DbErr> {
++ pub async fn commit(&mut self) -> Result<(), DbErr> {

-- pub async fn rollback(mut self) -> Result<(), DbErr> {
++ pub async fn rollback(&mut self) -> Result<(), DbErr> {

*/

use std::time::Duration;

use uuid::Uuid;

use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DatabaseTransaction, DbErr,
    JsonValue, QueryResult, Statement, TransactionTrait,
}; //, DbConn

async fn connect() -> Result<DatabaseConnection, DbErr> {
    
    let mysql_uri = "mysql://root:dummypass@localhost:3306/Test01DB";

    let mut opt = ConnectOptions::new(mysql_uri);
    opt.max_connections(10)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);

    let db = Database::connect(opt).await?;
    //let db = Database::connect(mysql_uri).await?;

    Ok(db)
}

async fn begin(db: &mut DatabaseConnection) -> Result<DatabaseTransaction, DbErr> {
    Ok(db.begin().await?)
}

async fn commit(tx: &mut DatabaseTransaction) -> Result<(), DbErr> {
    tx.commit().await?;

    Ok(())
}

#[allow(dead_code)]
async fn rollback(tx: &mut DatabaseTransaction) -> Result<(), DbErr> {
    tx.rollback().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    //-> Result<(),Box<dyn Error + 'static>> {

    let mut db_conn = connect().await?;

    let mut db_tx = begin(&mut db_conn).await?;

    let id = Uuid::new_v4();

    let insert_sql = "Insert Into simpleTest(Id,Name) Values('".to_string()
        + &id.to_string()
        + "','"
        + &id.to_string()
        + "')";

    db_tx.execute_unprepared(&insert_sql).await?;

    commit(&mut db_tx).await?;

    let query_res_vec: Vec<JsonValue> =
        <JsonValue as sea_orm::FromQueryResult>::find_by_statement(Statement::from_sql_and_values(
            db_conn.get_database_backend(),
            "SELECT * FROM `simpleTest`;",
            [],
        ))
        .all(&db_conn)
        .await?;

    for query_result in query_res_vec {
        println!("Id => {}", query_result["Id"]);
        println!("Name => {}", query_result["Name"]);
    }

    let query_res_vec: Vec<QueryResult> = db_conn
        .query_all(Statement::from_string(
            db_conn.get_database_backend(), //DatabaseBackend::MySql,
            "SELECT * FROM `simpleTest`;",
        ))
        .await?;

    for query_result in query_res_vec {
        let id = query_result.try_get::<String>("", "Id");

        println!("Id => {}", id.unwrap());
    }

    //println!("Hello, world! {}", id.to_string());

    Ok(())
}
