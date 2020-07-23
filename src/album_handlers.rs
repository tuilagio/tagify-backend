use crate::album_models::{AlbumsPreview, CreateAlbum, TagPhoto, UpdateAlbum, VerifyPhoto, Search};
use crate::user_models::User;
use crate::gg_storage;
extern crate reqwest;

use crate::errors::{DBError, HandlerError};
use crate::my_identity_service::Identity;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Result};
use deadpool_postgres::Pool;
use log::{error, info};
use std::fs;

use crate::db;

pub async fn create_album(
    pool: web::Data<Pool>,
    data: web::Json<CreateAlbum>,
    id: Identity,
    tagify_albums_path: web::Data<String>,
    gg_storage_data: web::Data<gg_storage::GoogleStorage>,
) -> Result<HttpResponse, HandlerError> {
    let user: User = id.identity();

    let bearer_string = &gg_storage_data.bearer_string;
    let key_refresh_token = &gg_storage_data.key_refresh_token;
    let project_number = &gg_storage_data.project_number;
    let google_storage_enable = &gg_storage_data.google_storage_enable;

    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
    };
    //create album without tags
    let result = match db::create_album(&client, &data, user.id).await {
        Err(e) => {
            error!("Error occured after create_album: {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(album) => {
            print!("google_storage_enable {:?}", google_storage_enable.to_string());
            if google_storage_enable.to_string() == "true" {
                let client_r = reqwest::Client::new();
                let bucket_name: String = format!("{}{}", gg_storage::PREFIX_BUCKET, &album.id);
                let response = gg_storage::create_bucket(
                    &client_r, &bearer_string.to_string(), &key_refresh_token.to_string(), 
                    &project_number.to_string(), &bucket_name).await;
                match response {
                    Ok(response) => {
                        if response.contains("error") {
                            error!("Fail creating google storage bucket: {}", response);
                            // Delete created album in db because creating on gg storage failed:
                            match db::delete_album(&client, album.id).await {
                                Err(e) => {
                                    error!("Error occured deleting album: {}", e);
                                    return Err(HandlerError::InternalError);
                                }
                                Ok(_) => {
                                    return Err(HandlerError::InternalError);
                                }
                            };
                        }
                        album
                    },
                    Err(e) => {
                        error!("Fail creating google storage bucket: {}", e);
                        return Err(HandlerError::InternalError);
                    }
                }
            } else {
                let path = format!("{}{}", tagify_albums_path.to_string(), &album.id);
                match std::fs::create_dir_all(&path) {
                    Ok(_) => info!("Created folder for album with id={}", &album.id),
                    Err(e) => {
                        error!(
                            "Error creating folder for album with id={}: {:?}",
                            &album.id, e
                        );
                        return Err(HandlerError::InternalError);
                    }
                }
                album
            }
        }
    };

    Ok(HttpResponse::build(StatusCode::OK).json(result))
}

pub async fn get_own_albums(
    pool: web::Data<Pool>,
    id: Identity,
) -> Result<HttpResponse, HandlerError> {
    let user: User = id.identity();

    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let result = match db::get_users_albums(&client, user.id).await {
        Err(e) => {
            error!("Error occured get users albums: {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(item) => item,
    };

    Ok(HttpResponse::build(StatusCode::OK).json(result))
}

pub async fn get_album_by_id(
    pool: web::Data<Pool>,
    album_id: web::Path<(i32,)>,
) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let result = match db::get_album_by_id(&client, album_id.0).await {
        Err(e) => {
            error!("Error occured : {}", e);
            if let DBError::BadArgs{err} = e {
                return Err(HandlerError::BadClientData{field: err});
            }

            return Err(HandlerError::InternalError);
        }
        Ok(item) => item,
    };

    Ok(HttpResponse::build(StatusCode::OK).json(result))
}

// gets all albums data (id, title, description, first_photo)
pub async fn get_all_albums(pool: web::Data<Pool>) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let albums: AlbumsPreview = match db::get_all_albums(client).await {
        Ok(albums) => albums,
        Err(e) => match e {
            DBError::PostgresError(e) => {
                error!("Getting albums failed {}", e);
                return Err(HandlerError::AuthFail);
            }
            DBError::MapperError(e) => {
                error!("Error occured: {}", e);
                return Err(HandlerError::InternalError);
            }
            DBError::ArgonError(e) => {
                error!("Error occured: {}", e);
                return Err(HandlerError::InternalError);
            }
            DBError::BadArgs { err } => {
                error!("Error occured: {}", err);
                return Err(HandlerError::BadClientData {
                    field: err.to_owned(),
                });
            }
        },
    };
    Ok(HttpResponse::build(StatusCode::OK).json(albums))
}

// get 20 next photos from album (start at 20 * index)
pub async fn get_photos_from_album(
    pool: web::Data<Pool>,
    data: web::Path<(i32, i32)>,
) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let result = match db::get_photos_from_album(&client, &data.0, &data.1).await {
        Err(e) => {
            error!("Error occured : {}", e);
            if let DBError::BadArgs{err} = e {
                return Err(HandlerError::BadClientData{field: err});
            }

            return Err(HandlerError::InternalError);
        }
        Ok(item) => item,
    };

    Ok(HttpResponse::build(StatusCode::OK).json(result))
}

pub async fn delete_album_by_id(
    pool: web::Data<Pool>,
    album_id: web::Path<(i32,)>,
    id: Identity,
    tagify_albums_path: web::Data<String>,
    gg_storage_data: web::Data<gg_storage::GoogleStorage>,
) -> Result<HttpResponse, HandlerError> {
    let user: User = id.identity();

    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let result = match db::get_album_by_id(&client, album_id.0).await {
        // TODO: Error response for case supplied album id not found?
        Err(e) => {
            error!("Error occured get user's album: {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(item) => item,
    };

    if user.id == result.users_id || user.role == "admin" {
        // Delete album from DB
        match db::delete_album(&client, album_id.0).await {
            Err(e) => {
                error!("Error occured: {}", e);
                return Err(HandlerError::InternalError);
            }
            Ok(_) => {
                // DELETE from storage:
                if gg_storage_data.google_storage_enable.to_string() == "true" {
                    //  Google storage
                    let client_r = reqwest::Client::new();
                    let bearer_string = &gg_storage_data.bearer_string;

                    let bucket_name: String = format!("{}{}", gg_storage::PREFIX_BUCKET, &album_id.0);
                    match gg_storage::delete_bucket(&client_r, &bearer_string.to_string(), &bucket_name)
                    .await {
                        Err(e) => {
                            error!("Error occured deleting album from google storage: {}", e);
                            return Err(HandlerError::InternalError);
                        }
                        Ok(response) => {
                            if response.contains("error") {
                                // This error is considered "acceptable"
                                error!("Fail deleting google storage bucket: {}", response);
                            }
                        }
                    }
                } else {
                    // Local
                    let dir_path = format!("{}{}", &tagify_albums_path.to_string(), &album_id.0);
                    match fs::remove_dir_all(&dir_path) {
                        Err(e) => {
                            error!("Error occured deleting album {} from local storage: {}", dir_path, e);
                            return Err(HandlerError::InternalError);
                        },
                        Ok(_) => {}
                    }
                }
            },
        };
    } else {
        return Err(HandlerError::PermissionDenied{err_message: "You are not the owner of this album".to_string()});
    }
    Ok(HttpResponse::new(StatusCode::OK))
}

pub async fn update_album_by_id(
    pool: web::Data<Pool>,
    album_id: web::Path<(i32,)>,
    id: Identity,
    data: web::Json<UpdateAlbum>,
) -> Result<HttpResponse, HandlerError> {
    let user: User = id.identity();

    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let result = match db::get_album_by_id(&client, album_id.0).await {
        Err(e) => {
            error!("Error occured get users albums: {}", e);

            return Err(HandlerError::InternalError);
        }
        Ok(item) => item,
    };

    if user.id == result.users_id || user.role == "admin" {
        match db::update_album(&client, album_id.0, &data).await {
            Err(e) => {
                error!("Error occured: {}", e);
                return Err(HandlerError::InternalError);
            }
            Ok(num_updated) => num_updated,
        };
    } else {
        return Err(HandlerError::PermissionDenied{err_message: "You are not the owner of this album".to_string()});
    }
    Ok(HttpResponse::new(StatusCode::OK))
}

// tag photo + set coordinates
pub async fn tag_photo_by_id(
    pool: web::Data<Pool>,
    data_id: web::Path<(i32,)>,
    data: web::Json<TagPhoto>,
) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let is_success = match db::tag_photo_by_id(client, &data_id.0, &data).await {
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(item) => item
    };


    if is_success {
        Ok(HttpResponse::build(StatusCode::OK).finish())
    } else {
        error!("Error occured : timeout");
        Err(HandlerError::Timeout)
    }

}

// verify_photo
pub async fn verify_photo_by_id(
    pool: web::Data<Pool>,
    data_id: web::Path<(i32,)>,
    data: web::Json<VerifyPhoto>,
) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    match db::verify_photo_by_id(client, &data_id.0, data.verified).await {
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(item) => match item {
            true => return Ok(HttpResponse::build(StatusCode::OK).json(item)),
            false => {
                error!("Error occured : timeout");
                return Err(HandlerError::BadClientData {
                    field: "timeout".to_string(),
                });
            }
        },
    };
}

// get next 20 photos for tagging
pub async fn get_photos_for_tagging(
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

    let result = match db::get_photos_for_tagging(client, &data.0).await {
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(item) => item,
    };

    Ok(HttpResponse::build(StatusCode::OK).json(result))
}

pub async fn search(
    pool: web::Data<Pool>,
    data: web::Json<Search>
) -> Result<HttpResponse, HandlerError> {

    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let albums: AlbumsPreview = match db::get_searched_albums(client, &data.search_after).await {
        Ok(albums) => albums,
        Err(e) => match e {
            DBError::PostgresError(e) => {
                error!("Getting albums failed {}", e);
                return Err(HandlerError::AuthFail);
            }
            DBError::MapperError(e) => {
                error!("Error occured: {}", e);
                return Err(HandlerError::InternalError);
            }
            DBError::ArgonError(e) => {
                error!("Error occured: {}", e);
                return Err(HandlerError::InternalError);
            }
            DBError::BadArgs { err } => {
                error!("Error occured: {}", err);
                return Err(HandlerError::BadClientData {
                    field: err.to_owned(),
                });
            }
        },
    };
    


    Ok(HttpResponse::build(StatusCode::OK).json(albums))
}