use crate::errors::HandlerError;
use crate::gg_storage;
use crate::user_models::{
    CreateImageMeta, Hash, LoginData, SendUser, Status, UpdateUserNickname, UpdateUserPassword,
    User,
};
use actix_web::http::StatusCode;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use deadpool_postgres::Pool;

extern crate reqwest;
use crate::album_models::Album;
use crate::db;
use crate::errors;
use crate::my_cookie_policy::MyCookieIdentityPolicy;
use crate::my_identity_service::{login_user, Identity};

use crate::utils;
use std::fs;
use std::io::Write;

// use bytes::Bytes;
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use log::{debug, error, info};

pub async fn status() -> Result<HttpResponse, HandlerError> {
    let status = String::from("server is working!");
    let status_message = Status { status: status };
    Ok(HttpResponse::build(StatusCode::OK).json(status_message))
}

pub async fn get_user(id: Identity) -> Result<HttpResponse, HandlerError> {
    // Get user identity
    let user: User = id.identity();

    let send_user = SendUser {
        id: user.id,
        username: user.username,
        nickname: user.nickname,
        role: user.role,
    };

    Ok(HttpResponse::build(StatusCode::OK).json(send_user))
}

pub async fn logout(id: Identity) -> Result<HttpResponse, HandlerError> {
    id.logout();

    Ok(HttpResponse::new(StatusCode::OK))
}

pub async fn login(
    data: web::Json<LoginData>,
    pool: web::Data<Pool>,
    req: HttpRequest,
    cookie_factory: web::Data<MyCookieIdentityPolicy>,
) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let user: User = match db::get_user_by_name(client, &data.username).await {
        Ok(user) => user,
        Err(e) => match e {
            errors::DBError::PostgresError(e) => {
                error!("Getting user failed: {}", e);
                return Err(HandlerError::AuthFail);
            }
            errors::DBError::MapperError(e) => {
                error!("Error occured: {}", e);
                return Err(HandlerError::InternalError);
            }
            errors::DBError::ArgonError(e) => {
                error!("Error occured: {}", e);
                return Err(HandlerError::InternalError);
            }
            errors::DBError::BadArgs { err } => {
                error!("Error occured: {}", err);
                return Err(HandlerError::BadClientData {
                    field: err.to_owned(),
                });
            }
        },
    };

    match user.verify_password(data.password.as_bytes()) {
        Ok(correct) => {
            if !correct {
                return Err(HandlerError::AuthFail);
            }
        }
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
    }

    debug!("User {} logged in successfully", user.username);
    Ok(login_user(req, cookie_factory.get_ref(), user).await)
}

pub async fn update_user_password(
    pool: web::Data<Pool>,
    id: Identity,
    data: web::Json<UpdateUserPassword>,
) -> Result<HttpResponse, HandlerError> {
    // Get user identity
    let user: User = id.identity();

    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let new_user = User {
        id: user.id,
        username: user.username,
        nickname: user.nickname,
        password: data.password.clone(),
        role: user.role,
    };

    let result = db::update_user_password(&client, &new_user).await;

    match result {
        Err(e) => match e {
            errors::DBError::PostgresError(e) => {
                error!("Getting user failed: {}", e);
                return Err(HandlerError::InternalError);
            }
            errors::DBError::MapperError(e) => {
                error!("Error occured: {}", e);
                return Err(HandlerError::InternalError);
            }
            errors::DBError::ArgonError(e) => {
                error!("Error occured: {}", e);
                return Err(HandlerError::InternalError);
            }
            errors::DBError::BadArgs { err } => {
                error!("Error occured: {}", err);
                return Err(HandlerError::BadClientData {
                    field: err.to_owned(),
                });
            }
        },
        Ok(num_updated) => num_updated,
    };

    Ok(HttpResponse::new(StatusCode::OK))
}

pub async fn update_user_nickname(
    pool: web::Data<Pool>,
    id: Identity,
    data: web::Json<UpdateUserNickname>,
) -> Result<HttpResponse, HandlerError> {
    // Get user identity
    let user: User = id.identity();

    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let new_user = User {
        id: user.id,
        username: user.username,
        nickname: data.nickname.clone(),
        password: user.password,
        role: user.role,
    };

    let result = db::update_user_nickname(&client, &new_user).await;

    match result {
        Err(e) => match e {
            errors::DBError::PostgresError(e) => {
                error!("Getting user failed: {}", e);
                return Err(HandlerError::InternalError);
            }
            errors::DBError::MapperError(e) => {
                error!("Error occured: {}", e);
                return Err(HandlerError::InternalError);
            }
            errors::DBError::ArgonError(e) => {
                error!("Error occured: {}", e);
                return Err(HandlerError::InternalError);
            }
            errors::DBError::BadArgs { err } => {
                error!("Error occured: {}", err);
                return Err(HandlerError::BadClientData {
                    field: err.to_owned(),
                });
            }
        },
        Ok(num_updated) => num_updated,
    };

    Ok(HttpResponse::new(StatusCode::OK))
}

pub async fn delete_user(
    pool: web::Data<Pool>,
    id: Identity,
) -> Result<HttpResponse, HandlerError> {
    // Get user identity
    let user: User = id.identity();

    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let result = db::delete_user(&client, user.id).await;

    match result {
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(num_updated) => num_updated,
    };

    Ok(HttpResponse::new(StatusCode::OK))
}

pub async fn post_photo(
    pool: web::Data<Pool>,
    tagify_albums_path: web::Data<String>,
    gg_storage_data: web::Data<gg_storage::GoogleStorage>,
    parameters: web::Path<(i32,)>,
    mut payload: Multipart,
    id: Identity,
) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
    };
    let user: User = id.identity();
    let album_id = parameters.0;
    let album_path = format!("{}{}/", tagify_albums_path.to_string(), &album_id);
    // For gg storage
    let bearer_string = &gg_storage_data.bearer_string;
    let google_storage_enable = &gg_storage_data.google_storage_enable;
    let client_r = reqwest::Client::new();
    let bucket_name: String = format!("{}{}", gg_storage::PREFIX_BUCKET, &album_id);

    // Check user has right to change file image:
    let mut result: Album = match db::get_album_by_id(&client, album_id).await {
        Err(e) => {
            error!("Error occured get users albums: {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(item) => item,
    };

    if user.id != result.users_id && user.role != "admin" {
        return Err(HandlerError::PermissionDenied {
            err_message: format!("Only owner can add image to album {}", album_id),
        });
    }

    // Check album exist
    if google_storage_enable.to_string() == "true" {
        match gg_storage::get_bucket(&client_r, &bearer_string, &bucket_name).await {
            Err(e) => {
                error!("Error occured getting bucket from gg storage: {}", e);
                return Err(HandlerError::InternalError);
            }
            Ok(response) => {
                if response.contains("error") {
                    return Err(HandlerError::BadClientData {
                        field: "Album not found in storage".to_string(),
                    });
                }
            }
        }
    } else {
        if !std::path::Path::new(&album_path).exists() {
            error!(
                "Error occured : album with id={} not found on disk",
                &album_id
            );
            return Err(HandlerError::BadClientData {
                field: "Album not found".to_string(),
            });
        }
    }
    if !db::check_album_exist_by_id(&client, &album_id).await {
        error!(
            "Error occured : album with id={} not found in db",
            &album_id
        );
        return Err(HandlerError::BadClientData {
            field: "Album not found".to_string(),
        });
    }

    while let Ok(Some(mut field)) = payload.try_next().await {
        // Get list of names from db and storage
        let mut filenames_storage: Vec<String> = Vec::new();
        let filenames_db: Vec<String> =
            db::get_image_filenames_of_album_with_id(&client, &album_id).await;
        if google_storage_enable.to_string() == "true" {
            match gg_storage::get_all_object_names_from_bucket(
                &client_r,
                &bearer_string,
                &bucket_name,
            )
            .await
            {
                Err(e) => {
                    filenames_storage.push("filenames_storage not used here".to_string());
                    error!(
                        "Error occured : album with id={} not found in google storage {:?}",
                        &album_id, e
                    );
                    return Err(HandlerError::BadClientData {
                        field: "Album not found in storage".to_string(),
                    });
                }
                Ok(objectnames_bucket) => {
                    filenames_storage = objectnames_bucket;
                }
            }
        } else {
            filenames_storage = utils::get_filenames_in_folder(&album_path);
        }

        // Create new name for object
        let new_filename = utils::calculate_next_filename_image(&filenames_storage, &filenames_db);

        let content_type = field.content_disposition().unwrap();
        let filename_original = content_type.get_filename().unwrap();
        let filename_clean = sanitize_filename::sanitize(&filename_original);
        let vec: Vec<&str> = filename_clean.split(".").collect();
        if vec.len() < 2 {
            info!(
                "Filename {} in payload has no extension. Skip.",
                filename_original
            );
            continue;
        }
        let file_extension = vec[vec.len() - 1];
        let new_filename_with_ext = format!("{}.{}", new_filename, file_extension);

        // Save file to storage
        if google_storage_enable.to_string() == "true" {
            info!(
                "new_filename_with_ext gg storage: {}",
                &new_filename_with_ext
            );
            // TODO: multipart is not really understand here
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                match gg_storage::upload_buffer_with_name_to_bucket(
                    &client_r,
                    &bearer_string,
                    &bucket_name,
                    &new_filename_with_ext,
                    data,
                )
                .await
                {
                    Err(e) => {
                        error!("Error uploading object to google storage {:?}", &e);
                        return Err(HandlerError::InternalError);
                    }
                    Ok(response) => {
                        if response.contains("error") {
                            error!(
                                "Error occured : error uploading object to google storage {:?}",
                                &response
                            );
                            return Err(HandlerError::InternalError);
                        }
                    }
                }
            }
        } else {
            let filepath = format!("{}{}", album_path, new_filename_with_ext);
            // File::create is blocking operation, use threadpool
            // Write file
            let mut f = web::block(|| std::fs::File::create(filepath))
                .await
                .unwrap();
            // Field in turn is stream of *Bytes* object
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                // filesystem operations are blocking, we have to use threadpool
                f = match web::block(move || f.write_all(&data).map(|_| f)).await {
                    Ok(item) => item,
                    Err(e) => {
                        error!("Error occured : {}", e);
                        return Err(HandlerError::InternalError);
                    }
                }
            }
        }
        // Write to db
        let image_meta = match db::create_image_meta(
            &client,
            &CreateImageMeta {
                album_id: album_id.clone(),
                coordinates: "".to_string(),
                file_path: new_filename_with_ext.clone(),
            },
        )
        .await
        {
            Ok(i) => {
                info!(
                    "Write meta data for {} to db success under {}",
                    filename_original, &new_filename_with_ext
                );
                i
            }
            Err(e) => {
                error!("Write file meta to db failed: {:?}", e);
                return Err(HandlerError::InternalError);
            }
        };
        if result.first_photo.is_none() {
            result = match db::album_set_first_image(&client, album_id, Some(image_meta.id)).await {
                Ok(i) => i,
                Err(e) => {
                    error!("Error occured : {}", e);
                    return Err(HandlerError::InternalError);
                }
            }
        }
    }
    Ok(HttpResponse::build(StatusCode::OK).finish())
}

pub async fn put_photo(
    pool: web::Data<Pool>,
    tagify_albums_path: web::Data<String>,
    gg_storage_data: web::Data<gg_storage::GoogleStorage>,
    parameters: web::Path<(i32, i32)>,
    mut payload: Multipart,
    id: Identity,
) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
    };
    let user: User = id.identity();
    let album_id = parameters.0;
    let image_id = parameters.1;
    let album_path = format!("{}{}/", tagify_albums_path.to_string(), &album_id);

    // For gg storage
    let bearer_string = &gg_storage_data.bearer_string;
    let google_storage_enable = &gg_storage_data.google_storage_enable;
    let client_r = reqwest::Client::new();
    let bucket_name: String = format!("{}{}", gg_storage::PREFIX_BUCKET, &album_id);

    // Check user has right to change file image:
    let result = match db::get_album_by_id(&client, album_id).await {
        Err(e) => {
            error!("Error occured get users albums: {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(item) => item,
    };

    if user.id != result.users_id && user.role != "admin" {
        return Err(HandlerError::PermissionDenied {
            err_message: format!("Only owner can add image to album {}", album_id),
        });
    }

    // Check album exist
    if google_storage_enable.to_string() == "true" {
        match gg_storage::get_bucket(&client_r, &bearer_string, &bucket_name).await {
            Err(e) => {
                error!("Error occured getting bucket from gg storage: {}", e);
                return Err(HandlerError::InternalError);
            }
            Ok(response) => {
                if response.contains("error") {
                    return Err(HandlerError::BadClientData {
                        field: "Album not found in storage".to_string(),
                    });
                }
            }
        }
    } else {
        if !std::path::Path::new(&album_path).exists() {
            error!(
                "Error occured : album with id={} not found on disk",
                &album_id
            );
            return Err(HandlerError::BadClientData {
                field: "Album not found".to_string(),
            });
        }
    }
    if !db::check_album_exist_by_id(&client, &album_id).await {
        error!(
            "Error occured : album with id={} not found in db",
            &album_id
        );
        return Err(HandlerError::BadClientData {
            field: "Album not found".to_string(),
        });
    }

    // Check if image exists in db:
    let file_path_db =
        db::get_image_file_path_with_id_from_album(&client, &album_id, &image_id).await;
    if file_path_db == "".to_string() {
        return Err(HandlerError::BadClientData {
            field: "Id of image not found in db".to_string(),
        });
    }

    while let Ok(Some(mut field)) = payload.try_next().await {
        // Extract num part of filename from db
        let mut new_filename = "".to_string();
        let vec: Vec<&str> = file_path_db.split(".").collect();
        let fname: &str = vec[0];
        if fname.parse::<u32>().is_ok() {
            new_filename = fname.parse().unwrap();
        }

        let content_type = field.content_disposition().unwrap();
        let filename_original = content_type.get_filename().unwrap();
        let filename_clean = sanitize_filename::sanitize(&filename_original);
        let vec: Vec<&str> = filename_clean.split(".").collect();
        if vec.len() < 2 {
            info!(
                "Filename {} in payload has no extension. Skip.",
                filename_original
            );
            continue;
        }
        let file_extension = vec[vec.len() - 1];
        let new_filename_with_ext = format!("{}.{}", new_filename, file_extension);
        let filepath = format!("{}{}", album_path, new_filename_with_ext);

        // Save file to storage
        if google_storage_enable.to_string() == "true" {
            // Delete old object (think of 1.png vs 1.jpg)
            match gg_storage::delete_object_from_bucket(
                &client_r,
                &bearer_string,
                &bucket_name,
                &file_path_db,
            )
            .await
            {
                Err(e) => {
                    error!("Error deleting object from google storage {:?}", &e);
                }
                Ok(_) => {}
            };
            // No need to delete object if existing in bucket.

            // Upload new object
            // TODO: multipart is not really understand here
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                match gg_storage::upload_buffer_with_name_to_bucket(
                    &client_r,
                    &bearer_string,
                    &bucket_name,
                    &new_filename_with_ext,
                    data,
                )
                .await
                {
                    Err(e) => {
                        error!("Error PUTting object to google storage {:?}", &e);
                        return Err(HandlerError::InternalError);
                    }
                    Ok(response) => {
                        if response.contains("error") {
                            error!(
                                "Error occured : error PUTting object to google storage {:?}",
                                &response
                            );
                            return Err(HandlerError::InternalError);
                        }
                    }
                }
            }
        } else {
            // Delete old file:
            let old_file_path = format!("{}{}", album_path, file_path_db);
            match fs::remove_file(&old_file_path) {
                Ok(_) => info!("Deleted old file {}", filepath),
                Err(e) => {
                    error!("Error deleting old file {}: {:?}", &old_file_path, e);
                    return Err(HandlerError::InternalError);
                }
            }
            // Delete existing file with new name:
            let new_file_path = format!("{}{}", album_path, new_filename_with_ext);
            if !std::path::Path::new(&filepath).exists() {
                match fs::remove_file(&new_file_path) {
                    Ok(_) => info!("Deleted existing file {}", new_file_path),
                    Err(e) => {
                        error!("Error deleting existing file {}: {:?}", &new_file_path, e);
                        return Err(HandlerError::InternalError);
                    }
                }
            }
            // File::create is blocking operation, use threadpool
            // Write file
            let mut f = web::block(|| std::fs::File::create(new_file_path))
                .await
                .unwrap();
            // Field in turn is stream of *Bytes* object
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                // filesystem operations are blocking, we have to use threadpool
                f = match web::block(move || f.write_all(&data).map(|_| f)).await {
                    Ok(item) => item,
                    Err(e) => {
                        error!("Error occured : {}", e);
                        return Err(HandlerError::InternalError);
                    }
                }
            }
        }

        // Update to db
        match db::update_image_meta(
            &client,
            &CreateImageMeta {
                album_id: album_id.clone(),
                coordinates: "".to_string(),
                file_path: new_filename_with_ext.clone(),
            },
            &image_id,
        )
        .await
        {
            Ok(_) => info!(
                "Update meta data for {} to db success under {}",
                filename_original, &new_filename_with_ext
            ),
            Err(e) => {
                error!("Update file meta to db failed: {:?}", e);
                return Err(HandlerError::InternalError);
            }
        };
    }
    Ok(HttpResponse::build(StatusCode::OK).finish())
}

pub async fn get_photo(
    pool: web::Data<Pool>,
    tagify_albums_path: web::Data<String>,
    gg_storage_data: web::Data<gg_storage::GoogleStorage>,
    parameters: web::Path<(i32, i32)>,
) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let album_id = parameters.0;
    let image_id = parameters.1;
    let album_path = format!("{}{}/", tagify_albums_path.to_string(), &album_id);

    // For gg storage
    let bearer_string = &gg_storage_data.bearer_string;
    let google_storage_enable = &gg_storage_data.google_storage_enable;
    let client_r = reqwest::Client::new();
    let bucket_name: String = format!("{}{}", gg_storage::PREFIX_BUCKET, &album_id);

    // Check album exist
    if google_storage_enable.to_string() == "true" {
        match gg_storage::get_bucket(&client_r, &bearer_string, &bucket_name).await {
            Err(e) => {
                error!("Error occured getting bucket from gg storage: {}", e);
                return Err(HandlerError::InternalError);
            }
            Ok(response) => {
                if response.contains("error") {
                    return Err(HandlerError::BadClientData {
                        field: "Album not found in storage".to_string(),
                    });
                }
            }
        }
    } else {
        if !std::path::Path::new(&album_path).exists() {
            error!(
                "Error occured : album with id={} not found on disk",
                &album_id
            );
            return Err(HandlerError::BadClientData {
                field: "Album not found".to_string(),
            });
        }
    }
    if !db::check_album_exist_by_id(&client, &album_id).await {
        error!(
            "Error occured : album with id={} not found in db",
            &album_id
        );
        return Err(HandlerError::BadClientData {
            field: "Album not found".to_string(),
        });
    }

    // Check if image exists in db:
    let file_path_db =
        db::get_image_file_path_with_id_from_album(&client, &album_id, &image_id).await;
    if file_path_db == "".to_string() {
        return Err(HandlerError::BadClientData {
            field: format!("Image with id={} of album id={} not found in db.\nImage not exists or false album id?", &image_id, &album_id).to_string()
        });
    }

    let filepath = format!("{}{}", album_path, file_path_db);
    let vec: Vec<&str> = file_path_db.split(".").collect();
    let file_ext: &str = vec[1];
    // Get image
    if google_storage_enable.to_string() == "true" {
        let bytes = gg_storage::download_object_bytes_from_bucket(
            &client_r,
            &bearer_string,
            &bucket_name,
            &file_path_db,
        )
        .await;
        let bb = match bytes {
            Err(e) => {
                error!("Error downloading object from google storage {:?}", &e);
                return Err(HandlerError::InternalError);
            }
            Ok(b) => b,
        };
        Ok(HttpResponse::build(StatusCode::OK)
            .content_type(format!("image/{}", file_ext))
            .body(bb))
    } else {
        // Check file exist
        if !std::path::Path::new(&filepath).exists() {
            error!(
                "Error occured : Image file with id={} not found on disk",
                &filepath
            );
            return Err(HandlerError::BadClientData {
                field: format!("File {} not found on disk", filepath).to_string(),
            });
        }

        // let mut bb: Vec<u8> = Vec::new();
        let bb: Vec<u8> = match std::fs::read(filepath) {
            Err(e) => {
                error!("Error openning local file {:?}", &e);
                return Err(HandlerError::InternalError);
            }
            Ok(bytes) => bytes,
        };
        Ok(HttpResponse::build(StatusCode::OK)
            .content_type(format!("image/{}", file_ext))
            .body(bb))
    }
}

#[allow(unused_assignments)]
pub async fn delete_photo(
    pool: web::Data<Pool>,
    tagify_albums_path: web::Data<String>,
    gg_storage_data: web::Data<gg_storage::GoogleStorage>,
    parameters: web::Path<(i32, i32)>,
    id: Identity,
) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
    };
    let user: User = id.identity();
    let album_id = parameters.0;
    let image_id = parameters.1;
    let album_path = format!("{}{}/", tagify_albums_path.to_string(), &album_id);

    // For gg storage
    let bearer_string = &gg_storage_data.bearer_string;
    let google_storage_enable = &gg_storage_data.google_storage_enable;
    let client_r = reqwest::Client::new();
    let bucket_name: String = format!("{}{}", gg_storage::PREFIX_BUCKET, &album_id);

    // Check user has right to change file image:
    let mut result: Album = match db::get_album_by_id(&client, album_id).await {
        Err(e) => {
            error!("Error occured get users albums: {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(item) => item,
    };

    if user.id != result.users_id && user.role != "admin" {
        return Err(HandlerError::PermissionDenied {
            err_message: format!("Only owner can delete image from album {}", album_id),
        });
    }

    // Check if image exists in db:
    let file_path_db =
        db::get_image_file_path_with_id_from_album(&client, &album_id, &image_id).await;
    if file_path_db == "".to_string() {
        return Err(HandlerError::BadClientData {
            field: format!("Id {} of image not found in db", image_id),
        });
    }

    // Check album exist
    if google_storage_enable.to_string() == "true" {
        match gg_storage::get_bucket(&client_r, &bearer_string, &bucket_name).await {
            Err(e) => {
                error!("Error occured getting bucket from gg storage: {}", e);
                return Err(HandlerError::InternalError);
            }
            Ok(response) => {
                if response.contains("error") {
                    return Err(HandlerError::BadClientData {
                        field: "Album not found in storage".to_string(),
                    });
                }
            }
        }
    } else {
        if !std::path::Path::new(&album_path).exists() {
            error!(
                "Error occured : album with id={} not found on disk",
                &album_id
            );
            return Err(HandlerError::BadClientData {
                field: "Album not found".to_string(),
            });
        }
    }
    if !db::check_album_exist_by_id(&client, &album_id).await {
        error!(
            "Error occured : album with id={} not found in db",
            &album_id
        );
        return Err(HandlerError::BadClientData {
            field: "Album not found".to_string(),
        });
    }

    // Delete file from storage
    if google_storage_enable.to_string() == "true" {
        match gg_storage::delete_object_from_bucket(
            &client_r,
            &bearer_string,
            &bucket_name,
            &file_path_db,
        )
        .await
        {
            Err(e) => {
                error!("Error deleting object from google storage {:?}", &e);
            }
            Ok(_) => {}
        };
    } else {
        // Check file exist
        let filepath = format!("{}{}", album_path, file_path_db);
        if !std::path::Path::new(&filepath).exists() {
            error!(
                "Error occured : image file with id={} not found on disk",
                &filepath
            );
            return Err(HandlerError::BadClientData {
                field: "File not found".to_string(),
            });
        }
        // Delete file
        match fs::remove_file(&filepath) {
            Ok(_) => info!("Deleted file "),
            Err(e) => {
                error!("Error deleting file {}: {:?}", &filepath, e);
                return Err(HandlerError::InternalError);
            }
        }
    }

    // Delete from db
    match db::delete_image_meta(&client, &image_id).await {
        Ok(_) => info!(
            "Delete meta id={} from album {} success",
            &image_id, &album_id
        ),
        Err(e) => {
            error!(
                "Delete meta id={} from album {} failed: {:?}",
                &image_id, &album_id, e
            );
            return Err(HandlerError::InternalError);
        }
    };

    if let Some(first_photo) = result.first_photo {
        debug!("first_photo: {} == image_id: {}", first_photo, image_id);
        if first_photo == image_id {
            let photo_id = match db::get_first_photo(&client, &album_id).await {
                Ok(i) => i,
                Err(e) => {
                    error!("Error occured : {}", e);
                    return Err(HandlerError::InternalError);
                }
            };
            debug!("photo_id: {:?}", photo_id);
            if let Some(p_id) = photo_id {
                result = match db::album_set_first_image(&client, album_id, Some(p_id)).await {
                    Ok(i) => i,
                    Err(e) => {
                        error!("Error occured : {}", e);
                        return Err(HandlerError::InternalError);
                    }
                }
            } else {
                result = match db::album_set_first_image(&client, album_id, None).await {
                    Ok(i) => i,
                    Err(e) => {
                        error!("Error occured : {}", e);
                        return Err(HandlerError::InternalError);
                    }
                }
            }
        }
    }

    Ok(HttpResponse::build(StatusCode::OK).finish())
}
