use crate::album_models::{Album, CreateAlbum, AlbumsPreview, AlbumPreview, PhotoPreview};
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
        "INSERT INTO albums (title, description, users_id, first_photo) VAlUES ($1, $2, $3, $4) RETURNING *",
        &[&album.title, &album.description, &id, &first_photo]).await?;
    // println!("restlt: {:?}", result);
    Ok(Album::from_row_ref(&result)?)
}

// get albums data to preview from DB
pub async fn get_all_albums(
    client: deadpool_postgres::Client
) -> Result<AlbumsPreview, DBError> {
    let mut albums = AlbumsPreview {
        albums: Vec::new()
    };
    
    for row in client.query("SELECT id, title, description, first_photo  FROM albums ", &[]).await? {
        let album = AlbumPreview {
            id: row.get(0),
            title: row.get(1),
            description: row.get(2),
            first_photo: row.get(3),
        };
        albums.albums.push(album);
    }
    Ok(albums)
}

// get all photos from certain album -> sort by date_created
pub async fn get_photos_from_album(
    client: deadpool_postgres::Client,
    id: &i32,
    index: &i32
) -> Result<Vec<PhotoPreview>, DBError> {
    let mut photos = Vec::new();

    let start_position = index * 20;
    let last_position = &start_position + 20;
    let mut current_position = 0;
    
    for row in client.query("SELECT id, file_path  FROM image_metas WHERE albums_id = $1 ", &[&id]).await? {
        if &current_position >= &start_position {
            let photo = PhotoPreview {
                id: row.get(0),
                file_path: row.get(1)
            };
            photos.push(photo);
            current_position = current_position + 1;
            if &current_position >= &last_position {
                break;
            }
        }else {
            current_position = current_position + 1;
        }
    }
    Ok(photos)
}


