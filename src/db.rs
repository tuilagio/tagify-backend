use crate::album_models::{Album, CreateAlbum};
use crate::errors::DBError;
use crate::user_models::{CreateUser, Hash, User, CreateImageMeta};

use actix_web::Result;
use tokio_pg_mapper::FromTokioPostgresRow;
use log::{error, info};

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
        "INSERT INTO albums (title, description, users_id, first_photo) VAlUES ($1, $2, $3, $4) RETURNING *",
        &[&album.title, &album.description, &id, &first_photo]).await?;
    // println!("restlt: {:?}", result);
    Ok(Album::from_row_ref(&result)?)
}

pub async fn create_image_meta (
    client: &deadpool_postgres::Client,
    image_meta: &CreateImageMeta,
) -> Result<bool, DBError> {
    let result = client.query_one(
        "insert into image_metas (albums_id, file_path, coordinates) values ($1, $2, $3) RETURNING *",
        &[&image_meta.albums_id, &image_meta.file_path, &image_meta.coordinates]).await?;
    // println!("restlt: {:?}", result);
    Ok(true)
}

pub async fn check_album_exist_by_id (
    client: &deadpool_postgres::Client,
    album_id: &i32,
) -> bool {
    let result = client.query_one(
        "SELECT * FROM albums WHERE id=$1", &[&album_id]).await;
    // println!("restlt: {:?}", result);
    match result {
        Ok(row) => return true,
        Err(e) => return false,
    }
}

pub async fn get_next_file_name_in_db (
    client: &deadpool_postgres::Client,
    album_id: &i32,
) -> u32 {

    let mut next: u32 = 1;
    //
    let result = client.query(
        "SELECT * FROM image_metas WHERE albums_id = $1 ORDER BY file_path DESC", &[&album_id]).await;
    match result {
        Ok(rows) => {
            if rows.len() == 0{
                info!("Album {} has no photo in db", album_id);
            } else {
                let k = rows.len() -1;
                for i in 0..k {
                    let file_path: String = rows[i].get(4);
                    // println!("{:?}", file_path);
                    let vec: Vec<&str> = file_path.split(".").collect();
                    let last_file_name: &str = vec[0];
                    if last_file_name.parse::<u32>().is_ok() {
                        let current: u32 = last_file_name.parse().unwrap();
                        next = current +1;
                        break;
                    }
                }
            }
        },
        Err(e) => {
            error!("Error getting get_next_file_name_in_db: {:?}", e);
        },
    }
    return next;
}

pub async fn get_image_filenames_of_album_with_id (
    client: &deadpool_postgres::Client,
    album_id: &i32,
) -> Vec<String> {

    let mut filenames_db = Vec::new();
    let result = client.query(
        "SELECT * FROM image_metas WHERE albums_id = $1 ORDER BY file_path DESC", &[&album_id]).await;
    match result {
        Ok(rows) => {
            if rows.len() == 0{
                info!("Album {} has no photo in db", album_id);
            } else {
                for row in rows {
                    let filename: String = row.get(4);
                    filenames_db.push(filename);
                }
            }
        },
        Err(e) => {
            error!("Error get_image_filenames_of_album_with_id: {:?}", e);
        },
    }
    return filenames_db;
}
