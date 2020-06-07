// use crate::models::Todo;
// use actix_web::Result;
// use deadpool_postgres::Client;
// use tokio_pg_mapper::FromTokioPostgresRow;

// use crate::errors::UserError;

// pub async fn get_all_todos_db(client: &Client) -> Result<Vec<Todo>, UserError> {
//     let statement = client.prepare("SELECT * from todos").await.unwrap();

//     let todos = client
//         .query(&statement, &[])
//         .await
//         .expect("ERROR GETTING TODO")
//         .iter()
//         .map(|row| Todo::from_row_ref(row).unwrap())
//         .collect::<Vec<Todo>>();
//     Ok(todos)
// }
