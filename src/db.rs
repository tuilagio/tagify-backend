use actix_web::Result;
use tokio_pg_mapper::FromTokioPostgresRow;
use crate::models::User;
use crate::errors;

pub async fn get_user(client: deadpool_postgres::Client, username: &str) -> Result<User, errors::DBError>{

   // Query data
   let result = client.query_one("SELECT * FROM users WHERE username = $1", &[&username]).await?;

   Ok(User::from_row_ref(&result)?)
}
