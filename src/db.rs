use crate::album_models::{
    Album, AlbumPreview, AlbumsPreview, CreateAlbum, PhotoPreview, PhotoToTag, TagPhoto,
    UpdateAlbum,
};
use crate::errors::DBError;
use crate::user_models::{CreateImageMeta, CreateUser, Hash, ImageMeta, SendUser, User};

use actix_web::Result;
use log::{debug, error, info};
use tokio_pg_mapper::FromTokioPostgresRow;

use chrono::offset::Utc;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

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

pub async fn update_user_nickname(
    client: &deadpool_postgres::Client,
    user: &User,
) -> Result<User, DBError> {
    let result = client
        .query_one(
            "UPDATE users SET nickname=$1 WHERE id=$2 RETURNING *",
            &[&user.nickname, &user.id],
        )
        .await?;
    Ok(User::from_row_ref(&result)?)
}

pub async fn update_user_password(
    client: &deadpool_postgres::Client,
    user: &User,
) -> Result<User, DBError> {
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
            "UPDATE users SET  password=$1 WHERE id=$2 RETURNING *",
            &[&hashed_pwd, &user.id],
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
) -> Result<Album, DBError> {
    let result = client.query_one(
        "INSERT INTO albums (title, description, tags, users_id, first_photo) VAlUES ($1, $2, $3, $4, NULL) RETURNING *",
        &[&album.title, &album.description, &album.tags, &id]).await?;
    // println!("restlt: {:?}", result);
    Ok(Album::from_row_ref(&result)?)
}

pub async fn create_image_meta(
    client: &deadpool_postgres::Client,
    image_meta: &CreateImageMeta,
) -> Result<ImageMeta, DBError> {
    let result = client.query_one(
        "insert into image_metas (album_id, file_path, coordinates, tag) values ($1, $2, $3, '') RETURNING *",
        &[&image_meta.album_id, &image_meta.file_path, &image_meta.coordinates]).await?;

    client
        .query(
            "UPDATE albums SET image_number = image_number +1 WHERE id = $1",
            &[&image_meta.album_id],
        )
        .await?;

    // println!("restlt: {:?}", result);
    Ok(ImageMeta::from_row_ref(&result)?)
}

pub async fn update_image_meta(
    client: &deadpool_postgres::Client,
    image_meta: &CreateImageMeta,
    image_id: &i32,
) -> Result<ImageMeta, DBError> {
    let result = client.query_one(
        "UPDATE image_metas SET album_id=$1, file_path=$2, coordinates=$3 WHERE id=$4 RETURNING *",
        &[&image_meta.album_id, &image_meta.file_path, &image_meta.coordinates, &image_id]).await?;
    // println!("restlt: {:?}", result);
    Ok(ImageMeta::from_row_ref(&result)?)
}

pub async fn delete_image_meta(
    client: &deadpool_postgres::Client,
    image_meta_id: &i32,
) -> Result<bool, DBError> {
    let result = client
        .query_one(
            "DELETE FROM image_metas WHERE id=$1 RETURNING album_id",
            &[&image_meta_id],
        )
        .await?;

    let album_id: i32 = result.get(0);
    client
        .query(
            "UPDATE albums SET image_number = image_number -1 WHERE id = $1",
            &[&album_id],
        )
        .await?;
    Ok(true)
}

pub async fn check_album_exist_by_id(client: &deadpool_postgres::Client, album_id: &i32) -> bool {
    let result = client
        .query_one("SELECT * FROM albums WHERE id=$1", &[&album_id])
        .await;
    // println!("restlt: {:?}", result);
    match result {
        Ok(_row) => return true,
        Err(e) => {
            error!("Error check_album_exist_by_id: {:?}", e);
            return false;
        }
    }
}

pub async fn get_image_filenames_of_album_with_id(
    client: &deadpool_postgres::Client,
    album_id: &i32,
) -> Vec<String> {
    let mut filenames_db: Vec<String> = Vec::new();
    let result = client
        .query(
            "SELECT * FROM image_metas WHERE album_id = $1 ORDER BY file_path DESC",
            &[&album_id],
        )
        .await;
    match result {
        Ok(rows) => {
            if rows.len() == 0 {
                info!("Album {} has no photo in db", album_id);
            } else {
                for row in rows {
                    let filename: String = row.get(3);
                    filenames_db.push(filename);
                }
            }
        }
        Err(e) => {
            error!("Error get_image_filenames_of_album_with_id: {:?}", e);
        }
    }
    return filenames_db;
}

// get albums data to preview from DB
pub async fn get_all_albums(client: deadpool_postgres::Client) -> Result<AlbumsPreview, DBError> {
    let mut albums = AlbumsPreview { albums: Vec::new() };

    for row in client
        .query(
            "SELECT id, title, description, first_photo  FROM albums ",
            &[],
        )
        .await?
    {
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

pub async fn get_first_photo(
    client: &deadpool_postgres::Client,
    id: &i32,
) -> Result<Option<i32>, DBError> {
    let row = client
        .query("SELECT id FROM image_metas WHERE album_id = $1", &[&id])
        .await?;
    if row.is_empty() {
        Ok(None)
    } else {
        Ok(row[0].get(0))
    }
}

// get all photos from certain album -> sort by date_created
pub async fn get_photos_from_album(
    client: &deadpool_postgres::Client,
    id: &i32,
    index: &i32,
) -> Result<Vec<PhotoPreview>, DBError> {
    let mut photos = Vec::new();

    let start_position = index * 20;
    let last_position = &start_position + 20;
    let mut current_position = 0;

    if check_album_exist_by_id(&client, &id).await == false {
        let err_str = format!("Album with id {} does not exist", id);
        return Err(DBError::BadArgs {
            err: err_str.to_string(),
        });
    }

    for row in client
        .query(
            "SELECT id, file_path  FROM image_metas WHERE album_id = $1 ",
            &[&id],
        )
        .await?
    {
        if &current_position >= &start_position {
            let photo = PhotoPreview {
                id: row.get(0),
                file_path: row.get(1),
            };
            photos.push(photo);
            current_position = current_position + 1;
            if &current_position >= &last_position {
                break;
            }
        } else {
            current_position = current_position + 1;
        }
    }
    Ok(photos)
}

pub async fn get_image_file_path_with_id_from_album(
    client: &deadpool_postgres::Client,
    album_id: &i32,
    image_id: &i32,
) -> String {
    let mut file_path = "".to_string();
    let result = client
        .query(
            "SELECT * FROM image_metas WHERE id = $1 AND album_id = $2",
            &[&image_id, &album_id],
        )
        .await;
    match result {
        Ok(rows) => {
            if rows.len() == 0 {
                info!("Image with id {} not found in db", image_id);
            } else {
                for row in rows {
                    file_path = row.get(3);
                    break;
                }
            }
        }
        Err(e) => {
            error!("Error get_image_file_path_with_id: {:?}", e);
        }
    }
    return file_path;
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
    client: &deadpool_postgres::Client,
    album_id: i32,
) -> Result<Album, DBError> {
    if check_album_exist_by_id(&client, &album_id).await == false {
        let err_str = format!("Album with id {} does not exist", album_id);
        return Err(DBError::BadArgs {
            err: err_str.to_string(),
        });
    }

    // Query data
    let result = client
        .query_one("SELECT * FROM albums WHERE id = $1", &[&album_id])
        .await?;
    println!("get_album_by_id {:?}", result);
    Ok(Album::from_row_ref(&result)?)
}

pub async fn delete_album(
    client: &deadpool_postgres::Client,
    album_id: i32,
) -> Result<Album, DBError> {
    client
        .query("DELETE FROM image_metas WHERE album_id = $1", &[&album_id])
        .await?; // need to delete all photos from the album firstly
    let result = client
        .query_one("DELETE FROM albums WHERE id=$1 RETURNING *", &[&album_id])
        .await?;
    Ok(Album::from_row_ref(&result)?)
}

pub async fn album_set_first_image(
    client: &deadpool_postgres::Client,
    album_id: i32,
    photo_id: Option<i32>,
) -> Result<Album, DBError> {
    let result = client
        .query_one(
            "UPDATE albums SET first_photo=$1 WHERE id=$2 RETURNING *",
            &[&photo_id, &album_id],
        )
        .await?;
    Ok(Album::from_row_ref(&result)?)
}

pub async fn update_album(
    client: &deadpool_postgres::Client,
    album_id: i32,
    album: &UpdateAlbum,
) -> Result<Album, DBError> {
    let result = client
        .query_one(
            "UPDATE albums SET title=$1, description=$2 WHERE id=$3 RETURNING *",
            &[&album.title, &album.description, &album_id],
        )
        .await?;
    Ok(Album::from_row_ref(&result)?)
}

pub async fn get_all_users(client: &deadpool_postgres::Client) -> Result<Vec<SendUser>, DBError> {
    let result = client
        .query("SELECT id, username, nickname, role FROM users ", &[])
        .await
        .expect("ERROR GETTING USERS")
        .iter()
        .map(|row| SendUser::from_row_ref(row).unwrap())
        .collect::<Vec<SendUser>>();

    Ok(result)
}

// tag photo + set coordinats
pub async fn tag_photo_by_id(
    client: deadpool_postgres::Client,
    photo_id: &i32,
    photo_data: &TagPhoto,
) -> Result<bool, DBError> {
    let current_time = Utc::now().timestamp();
    let offset: i64 = 30; // 30s timeout TODO: Change in prod to 15 min in sec

    let result = client
        .query_one("SELECT * FROM image_metas WHERE id = $1", &[&photo_id])
        .await?;
    let locked_at = result.get::<&str, i64>("locked_at");
    let album_id = result.get::<&str, i32>("album_id");

    debug!("Albumid: {}, Locked_at: {}", album_id, locked_at);
    if (&locked_at + &offset) > current_time {
        client
        .query(
            "UPDATE image_metas SET tag = $1, coordinates = $2, tagged = true, locked_at = 0 WHERE id = $3 ", // reset timer if tagged
            &[&photo_data.tag, &photo_data.coordinates, &photo_id],
        )
        .await?;

        client
            .query(
                "UPDATE albums SET tagged_number = tagged_number +1 WHERE id = $1",
                &[&album_id],
            )
            .await?;

        Ok(true)
    } else {
        Ok(false)
    }
}

// verify photo ( if true => set verify true, else delete tag and coordinates & set both verified and tagged as false)
pub async fn verify_photo_by_id(
    client: deadpool_postgres::Client,
    id: &i32,
    verified: bool,
) -> Result<bool, DBError> {
    let current_time = Utc::now().timestamp();
    let offset: i64 = 30; //15 min in sec

    let result = client
        .query_one("SELECT locked_at FROM image_metas WHERE id = $1", &[&id])
        .await?;
    if (&result.get(0) + &offset) > current_time {
        if verified {
            client
                .query(
                    "UPDATE image_metas SET verified = true, locked_at = 0 WHERE id = $1 ", // reset timer
                    &[&id],
                )
                .await?;
        } else {
            client
            .query(
                "UPDATE image_metas SET tag = '', coordinates = '', tagged = false, verified = false, locked_at = 0 WHERE id = $1 ", // reset timer
                &[ &id],
            )
            .await?;
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

//get photos for tagging
pub async fn get_photos_for_tagging(
    client: deadpool_postgres::Client,
    id: &i32,
) -> Result<Vec<PhotoToTag>, DBError> {
    let mut photos = Vec::new();

    let current_time = Utc::now().timestamp();
    let offset: i64 = 30; // 15 min in sec
    let time_after_offset: i64 = &current_time - &offset;

    for row in client.query("SELECT id, file_path, tagged, tag  FROM image_metas WHERE album_id = $1 AND verified = false AND locked_at <= $2", &[&id, &time_after_offset]).await? {
            let photo_timestamp = Utc::now();

            let photo = PhotoToTag {
                id: row.get(0),
                file_path: row.get(1),
                tagged: row.get(2),
                tag: row.get(3),
                timestamp: photo_timestamp
            };
            client.query("UPDATE image_metas SET locked_at = $2 WHERE id = $1 ", &[&&photo.id, &photo.timestamp.timestamp()]).await?;

            photos.push(photo);
            if photos.len() >= 20 {
                break;
            }
    }
    Ok(photos)
}

// get albums data to preview from DB but with fuzzy matcher
pub async fn get_searched_albums(
    client: deadpool_postgres::Client,
    search_after: &String,
) -> Result<AlbumsPreview, DBError> {
    let mut albums = AlbumsPreview { albums: Vec::new() };
    let matcher = SkimMatcherV2::default();
    let search = String::from(search_after);

    for row in client
        .query(
            "SELECT id, title, description, first_photo  FROM albums ",
            &[],
        )
        .await?
    {
        let album = AlbumPreview {
            id: row.get(0),
            title: row.get(1),
            description: row.get(2),
            first_photo: row.get(3),
        };

        if matcher.fuzzy_match(&album.title, &search).is_some() {
            albums.albums.push(album);
        }
    }
    Ok(albums)
}
