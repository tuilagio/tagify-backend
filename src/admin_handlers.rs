use crate::errors::HandlerError;
use crate::user_models::{CreateUser, UpdateUserAdmin, User, CreateImageMeta};
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Result};
use deadpool_postgres::Pool;
use log::{error, info};

use crate::db;
use crate::utils;
use std::io::Write;

use actix_multipart::Multipart;
use actix_web::{middleware, /* web, */ App, Error, /* HttpResponse, */ HttpServer};
use futures::{StreamExt, TryStreamExt};

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
            return Err(HandlerError::InternalError);
        }
        Ok(_res) => {}
    };

    Ok(HttpResponse::new(StatusCode::OK))
}

pub async fn post_photo(
    pool: web::Data<Pool>,
    tagify_albums_path: web::Data<String,>,
    album_id: web::Path<(i32,)>,
    mut payload: Multipart,
    // data: web::Json<CreateUser>,
) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            // return Err();
            return Err(HandlerError::InternalError);
        }
    };
    let album_path = format!("{}{}/", tagify_albums_path.to_string(), album_id.0);

    // Check album exist
    if !std::path::Path::new(&album_path).exists() || !db::check_album_exist_by_id(&client, &album_id.0).await {
        error!("Error occured : album with id={} not found on disk", album_id.0);
        return Err(HandlerError::InternalError);
    }
    // Check user has right to write: No need to do here because admin path
    
    while let Ok(Some(mut field)) = payload.try_next().await {
        // Get new filename base on data folder:
        let filename_folder: u32 = utils::get_next_file_name_in_folder(&album_path);
        // Get new filename base on db:
        let filename_db: u32 = db::get_next_file_name_in_db(&client, &album_id.0).await;
        println!("{} {} {}", album_path, filename_folder, filename_db);
        let filename_u32: u32 = if filename_db>filename_folder {
            filename_db
        } else {
            filename_folder
        };
        let content_type = field.content_disposition().unwrap();
        let filename_original = content_type.get_filename().unwrap();
        let filename_clean = sanitize_filename::sanitize(&filename_original);
        let vec: Vec<&str> = filename_clean.split(".").collect();
        let file_extension = vec[vec.len()-1];
        let filepath = format!("{}{}.{}", album_path, filename_u32, file_extension);
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
                    // return Err();
                    return Err(HandlerError::InternalError);
                }
            }
        }
        // Write to db
        match db::create_image_meta(
            &client, 
            &CreateImageMeta{
                albums_id: album_id.0, 
                coordinates: "".to_string(),
                file_path: format!("{}.{}", filename_u32, file_extension),
            }
        ).await {
            Ok(ok) => info!("Write to db success"),
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
    // data: web::Json<CreateUser>,
    parameters: web::Path<(i32, i32)>,
) -> Result<HttpResponse, HandlerError> {
    // let client = match pool.get().await {
    //     Ok(item) => item,
    //     Err(e) => {
    //         error!("Error occured : {}", e);
    //         return Err(HandlerError::InternalError);
    //     }
    // };

    // let result = match db::create_user(&client, &data).await {
    //     Err(e) => {
    //         error!("Error occured: {}", e);
    //         return Err(HandlerError::InternalError);
    //     }
    //     Ok(item) => item,
    // };
    println!("create_photo");

    Ok(HttpResponse::build(StatusCode::OK).json("result"))
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