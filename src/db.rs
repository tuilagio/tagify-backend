use crate::album_models::{Album, CreateAlbum};
use crate::errors::DBError;
use crate::user_models::{CreateUser, Hash, User};

use actix_web::Result;
use tokio_pg_mapper::FromTokioPostgresRow;

pub async fn get_user_by_name(
    client: deadpool_postgres::Client,
    username: &str,
) -> Result<User, DBError> {
    // Query data
    let result = client
        .query_one("SELECT * FROM users WHERE username = $1", &[&username])
        .await?;

    Ok(User::from_row_ref(&result)?)
}

pub async fn get_user(client: &deadpool_postgres::Client, id: i32) -> Result<User, DBError> {
    // Query data
    let result = client
        .query_one("SELECT * FROM users WHERE id = $1", &[&id])
        .await?;

    Ok(User::from_row_ref(&result)?)
}

pub async fn update_user(client: &deadpool_postgres::Client, user: &User) -> Result<User, DBError> {
    if user.password.len() < 4 {
        return Err(DBError::BadArgs {
            err: "Password is too short".to_owned(),
        });
    }

    let hashed_pwd = match user.get_hashed_password() {
        Ok(item) => item,
        Err(e) => return Err(DBError::ArgonError(e)),
    };

    let result = client
        .query_one(
            "UPDATE users SET nickname=$1, password=$2, role=$3 WHERE id=$4 RETURNING *",
            &[&user.nickname, &hashed_pwd, &user.role, &user.id],
        )
        .await?;
    Ok(User::from_row_ref(&result)?)
}

pub async fn create_user(
    client: &deadpool_postgres::Client,
    user: &CreateUser,
) -> Result<User, DBError> {
    if user.password.len() < 4 {
        return Err(DBError::BadArgs {
            err: "Password is too short".to_owned(),
        });
    }

    let hashed_pwd = user.get_hashed_password()?;

    let result = client.query_one(
      "INSERT INTO users (username, nickname, password, role) VAlUES ($1, $2, $3, $4) RETURNING *",
      &[&user.username, &user.nickname, &hashed_pwd, &user.role]).await?;
    Ok(User::from_row_ref(&result)?)
}

pub async fn delete_user(
    client: &deadpool_postgres::Client,
    user_id: i32,
) -> Result<User, DBError> {
    let result = client
        .query_one("DELETE FROM users WHERE id=$1 RETURNING *", &[&user_id])
        .await?;
    Ok(User::from_row_ref(&result)?)
}

//albums
pub async fn create_album(
    client: &deadpool_postgres::Client,
    album: &CreateAlbum,
    id: i32,
    first_photo: String,
) -> Result<Album, DBError> {
    let result = client.query_one(
        "INSERT INTO albums (title, description, tags, users_id, first_photo) VAlUES ($1, $2, $3, $4, $5) RETURNING *",
        &[&album.title, &album.description, &album.tags, &id, &first_photo]).await?;
    // println!("restlt: {:?}", result);
    Ok(Album::from_row_ref(&result)?)
}

pub async fn get_users_albums(
    client: &deadpool_postgres::Client,
    id: i32,
) -> Result<Vec<Album>, DBError> {
    let result = client
        .query("SELECT * FROM albums WHERE users_id = $1", &[&id])
        .await
        .expect("ERROR GETTING ALBUMS")
        .iter()
        .map(|row| Album::from_row_ref(row).unwrap())
        .collect::<Vec<Album>>();

    Ok(result)
}

pub async fn get_album_by_id(
    client: deadpool_postgres::Client,
    album_id: i32,
) -> Result<Album, DBError> {
    // Query data
    let result = client
        .query_one("SELECT * FROM albums WHERE id = $1", &[&album_id])
        .await?;

    Ok(Album::from_row_ref(&result)?)
}
