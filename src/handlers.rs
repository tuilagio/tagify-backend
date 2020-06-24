use crate::errors::HandlerError;
use crate::user_models::{
    Hash, LoginData, SendUser, Status, UpdateUser, 
    UpdateUserPassword, User, CreateImageMeta, UpdateUserNickname
};
use actix_web::http::StatusCode;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use deadpool_postgres::Pool;

use crate::db;
use crate::errors;
use crate::my_cookie_policy::MyCookieIdentityPolicy;
use crate::my_identity_service::{login_user, Identity};

use crate::utils;
use std::io::Write;
use std::fs;

use actix_multipart::Multipart;
use actix_web::{middleware, /* web, */ App, Error, /* HttpResponse, */ HttpServer};
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
    tagify_albums_path: web::Data<String,>,
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

    // Check user has right to write:
    let result = match db::get_album_by_id(&client, album_id).await {
        Err(e) => {
            error!("Error occured get users albums: {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(item) => item,
    };

    if user.id == result.users_id || user.role == "admin" {
        println!("usunie album");
        match db::delete_album(&client, album_id).await {
            Err(e) => {
                error!("Error occured: {}", e);
                return Err(HandlerError::InternalError);
            }
            Ok(result) => result,
        };
    } else {
        return Err(HandlerError::PermissionDenied {
            err_message: format!("Only owner can add image to album {}", album_id)
        });
    }

    // Check album exist
    if !std::path::Path::new(&album_path).exists() || !db::check_album_exist_by_id(&client, &album_id).await {
        error!("Error occured : album with id={} not found on disk", &album_id);
        return Err(HandlerError::InternalError);
    }
    
    while let Ok(Some(mut field)) = payload.try_next().await {

        let new_filename = utils::calculate_next_filename_image(
            &utils::get_filenames_in_folder(&album_path), 
            &db::get_image_filenames_of_album_with_id(&client, &album_id).await
        );

        let content_type = field.content_disposition().unwrap();
        let filename_original = content_type.get_filename().unwrap();
        let filename_clean = sanitize_filename::sanitize(&filename_original);
        let vec: Vec<&str> = filename_clean.split(".").collect();
        if vec.len() < 2 {
            info!("Filename {} in payload has no extension. Skip.", filename_original);
            continue;
        }
        let file_extension = vec[vec.len()-1];
        let new_filename_with_ext = format!("{}.{}", new_filename, file_extension);
        let filepath = format!("{}{}", album_path, new_filename_with_ext);
        println!("filepath: {}", filepath);
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
        // Write to db
        match db::create_image_meta(
            &client, 
            &CreateImageMeta{
                albums_id: album_id.clone(), 
                coordinates: "".to_string(),
                file_path: new_filename_with_ext.clone(),
            }
        ).await {
            Ok(_) => info!("Write meta data for {} to db success under {}", filename_original, &new_filename_with_ext),
            Err(e) => {
                error!("Write file meta to db failed: {:?}", e);
                return Err(HandlerError::InternalError);
            }
        };

    }
    Ok(HttpResponse::build(StatusCode::OK).json("Success write file(s)"))
}

pub async fn put_photo(
    pool: web::Data<Pool>,
    tagify_albums_path: web::Data<String,>,
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
            err_message: format!("Only owner can add image to album {}", album_id)
        });
    }

    // Check album exist
    if !std::path::Path::new(&album_path).exists() || !db::check_album_exist_by_id(&client, &album_id).await {
        error!("Error occured : album with id={} not found on disk", &album_id);
        return Err(HandlerError::InternalError);
    }

    // Check if image exists in db:
    let file_path = db::get_image_file_path_with_id(&client, &image_id).await;
    if file_path == "".to_string() {
        return Err(HandlerError::BadClientData {
            field: "Id of image not found in db".to_string()
        });
    }

    while let Ok(Some(mut field)) = payload.try_next().await {
        // Extract num part of filename from db
        let mut new_filename = "".to_string();
        let vec: Vec<&str> = file_path.split(".").collect();
        let fname: &str = vec[0];
        if fname.parse::<u32>().is_ok() {
            new_filename = fname.parse().unwrap();
        }

        let content_type = field.content_disposition().unwrap();
        let filename_original = content_type.get_filename().unwrap();
        let filename_clean = sanitize_filename::sanitize(&filename_original);
        let vec: Vec<&str> = filename_clean.split(".").collect();
        if vec.len() < 2 {
            info!("Filename {} in payload has no extension. Skip.", filename_original);
            continue;
        }
        let file_extension = vec[vec.len()-1];
        let new_filename_with_ext = format!("{}.{}", new_filename, file_extension);
        let filepath = format!("{}{}", album_path, new_filename_with_ext);
        println!("filepath: {}", filepath);

        // Delete old file:
        match fs::remove_file(&filepath) {
            Ok(_) => info!("Deleted file "),
            Err(e) => {
                error!("Error deleting file {}: {:?}", &filepath, e);
                return Err(HandlerError::InternalError);
            }
        }

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
        // Write to db
        match db::create_image_meta(
            &client, 
            &CreateImageMeta{
                albums_id: album_id.clone(), 
                coordinates: "".to_string(),
                file_path: new_filename_with_ext.clone(),
            }
        ).await {
            Ok(_) => info!("Write meta data for {} to db success under {}", filename_original, &new_filename_with_ext),
            Err(e) => {
                error!("Write file meta to db failed: {:?}", e);
                return Err(HandlerError::InternalError);
            }
        };

    }
    Ok(HttpResponse::build(StatusCode::OK).json("Success write file(s)"))
}

pub async fn get_photo(
    pool: web::Data<Pool>,
    // id: web::Path<(i32,)>,
    // data: web::Json<UpdateUserAdmin>,
    parameters: web::Path<(i32, i32)>,
) -> Result<HttpResponse, HandlerError> {
    println!("get_photo");
    Ok(HttpResponse::build(StatusCode::OK).json("result"))
}

pub async fn delete_photo(
    pool: web::Data<Pool>,
    parameters: web::Path<(i32, i32)>,
) -> Result<HttpResponse, HandlerError> {
    // let client = match pool.get().await {
    //     Ok(item) => item,
    //     Err(e) => {
    //         error!("Error occured: {}", e);
    //         return Err(HandlerError::InternalError);
    //     }
    // };

    // let result = db::delete_user(&client, data.0).await;

    // match result {
    //     Err(e) => {
    //         error!("Error occured: {}", e);
    //         return Err(HandlerError::InternalError);
    //     }
    //     Ok(_res) => {}
    // };
    println!("delete_photo");

    Ok(HttpResponse::new(StatusCode::OK))
}
