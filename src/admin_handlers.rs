use crate::errors::HandlerError;
use crate::user_models::{CreateUser, UpdateUserAdmin, User};
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Result};
use deadpool_postgres::Pool;
use log::{error, info};

use crate::db;
use crate::gg_storage;

use bytes::Bytes;
use std::fs;

pub async fn create_user(
    pool: web::Data<Pool>,
    data: web::Json<CreateUser>,
) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let result = match db::create_user(&client, &data).await {
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(item) => item,
    };

    Ok(HttpResponse::build(StatusCode::OK).json(result))
}

pub async fn update_user(
    pool: web::Data<Pool>,
    id: web::Path<(i32,)>,
    data: web::Json<UpdateUserAdmin>,
) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let user = match db::get_user(&client, id.0).await {
        Ok(i) => i,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::BadClientData {
                field: "User id does not exist".to_owned(),
            });
        }
    };

    let new_user = User {
        nickname: data.nickname.clone(),
        password: data.password.clone(),
        role: data.role.clone(),
        ..user
    };

    let result = match db::update_user(&client, &new_user).await {
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(item) => item,
    };

    Ok(HttpResponse::build(StatusCode::OK).json(result))
}

pub async fn delete_user(
    pool: web::Data<Pool>,
    data: web::Path<(i32,)>,
) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let result = db::delete_user(&client, data.0).await;

    match result {
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::BadClientData {
                field: "User id does not exist".to_owned(),
            });
        }
        Ok(_res) => {}
    };

    Ok(HttpResponse::new(StatusCode::OK))
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
    // let bearer_string = &gg_storage_data.bearer_string;
    let bearer_string: String = match fs::read_to_string("./credential/gen_token/oauth_key.txt") {
        Err(e) => {
            error!("Error reading oauth_key.txt  : {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(s) => s,
    };
    let client_r = reqwest::Client::new();
    let bucket_name: String = format!("{}{}", gg_storage::PREFIX_BUCKET, &album_id);

    // Check album exist
    if gg_storage_data.google_storage_enable {
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
    if gg_storage_data.google_storage_enable {
        let bytes = gg_storage::download_object_bytes_from_bucket(
            &client_r,
            &bearer_string,
            &bucket_name,
            &file_path_db,
        )
        .await;
        let mut _bb = Bytes::new();
        match bytes {
            Err(e) => {
                error!("Error downloading object from google storage {:?}", &e);
                return Err(HandlerError::InternalError);
            }
            Ok(b) => {
                _bb = b;
            }
        };
        Ok(HttpResponse::build(StatusCode::OK)
            .content_type(format!("image/{}", file_ext))
            .body(_bb))
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

        let mut _bb: Vec<u8> = Vec::new();
        match std::fs::read(filepath) {
            Err(e) => {
                error!("Error openning local file {:?}", &e);
                return Err(HandlerError::InternalError);
            }
            Ok(bytes) => {
                _bb = bytes;
            }
        };
        Ok(HttpResponse::build(StatusCode::OK)
            .content_type(format!("image/{}", file_ext))
            .body(_bb))
    }
}

pub async fn delete_photo(
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
    // let bearer_string = &gg_storage_data.bearer_string;
    let bearer_string: String = match fs::read_to_string("./credential/gen_token/oauth_key.txt") {
        Err(e) => {
            error!("Error reading oauth_key.txt  : {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(s) => s,
    };
    let client_r = reqwest::Client::new();
    let bucket_name: String = format!("{}{}", gg_storage::PREFIX_BUCKET, &album_id);

    // Check if image exists in db:
    let file_path_db =
        db::get_image_file_path_with_id_from_album(&client, &album_id, &image_id).await;
    if file_path_db == "".to_string() {
        return Err(HandlerError::BadClientData {
            field: "Id of image not found in db".to_string(),
        });
    }

    // Check album exist
    if gg_storage_data.google_storage_enable {
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
    if gg_storage_data.google_storage_enable {
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
                "Delete meta id={} from album {} success: {:?}",
                &image_id, &album_id, e
            );
            return Err(HandlerError::InternalError);
        }
    };

    Ok(HttpResponse::build(StatusCode::OK).json(format!("Success delete image id={}", &image_id)))
}

// get api/admin/users -> get all users data
// should i also list admin ?
pub async fn get_all_users(pool: web::Data<Pool>) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let result = match db::get_all_users(&client).await {
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(item) => item,
    };

    Ok(HttpResponse::build(StatusCode::OK).json(result))
}
