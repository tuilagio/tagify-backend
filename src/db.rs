use actix_web::Result;
use tokio_pg_mapper::FromTokioPostgresRow;
use crate::models::{User, InternalUser};
use crate::errors;

// TODO: improve coverage by also returning DBError

pub async fn get_user(client: deadpool_postgres::Client, username: &str) -> Result<User, errors::DBError>{

   // Query data
   let result = client.query_one("SELECT * FROM users WHERE username = $1", &[&username]).await?;

   Ok(User::from_row_ref(&result)?)
}

pub async fn get_internal_user(client: &deadpool_postgres::Client, username: &str) -> Result<InternalUser, errors::DBError>{
   let result = client.query_one("SELECT * FROM users WHERE username = $1", &[&username]).await?;
   Ok(InternalUser::from_row_ref(&result)?)
}

pub async fn get_internal_user_with_id(client: &deadpool_postgres::Client, user_id: &i32) -> Result<InternalUser, errors::DBError>{
   let result = client.query_one("SELECT * FROM users WHERE id = $1", &[&user_id]).await?;
   Ok(InternalUser::from_row_ref(&result)?)
}

/// Use with care: This function also updates username.
pub async fn update_user_with_id(client: &deadpool_postgres::Client, user_id: &i32, user: &User) -> Result<InternalUser, errors::DBError>{
   let result = client.query_one(
      "UPDATE users SET username=$1, nickname=$2, password=$3, role=$4 WHERE id=$5 RETURNING *", 
      &[&user.username, &user.nickname, &user.password, &user.role, &user_id]).await?;
   Ok(InternalUser::from_row_ref(&result)?)
}

pub async fn create_user(client: &deadpool_postgres::Client, user: &User) -> Result<InternalUser, errors::DBError>{
   let result = client.query_one(
      "INSERT INTO users (username, nickname, password, role) VAlUES ($1, $2, $3, $4) RETURNING *", 
      &[&user.username, &user.nickname, &user.password, &user.role]).await?;
   Ok(InternalUser::from_row_ref(&result)?)
}

pub async fn delete_user_with_id(client: &deadpool_postgres::Client, user_id: &i32) -> Result<InternalUser, errors::DBError>{
   let result = client.query_one(
      "DELETE FROM users WHERE id=$1 RETURNING *", 
      &[&user_id]).await?;
   Ok(InternalUser::from_row_ref(&result)?)
}

