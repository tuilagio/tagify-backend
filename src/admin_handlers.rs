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
    parameters: web::Path<(i32,)>,
    mut payload: Multipart,
) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
    };
    let album_id = parameters.0;
    let album_path = format!("{}{}/", tagify_albums_path.to_string(), &album_id);
    // Check album exist
    if !std::path::Path::new(&album_path).exists() || !db::check_album_exist_by_id(&client, &album_id).await {
        error!("Error occured : album with id={} not found on disk", &album_id);
        return Err(HandlerError::InternalError);
    }
    // Check user has right to write: No need to do here because admin path
    
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
    // Check album exist
    if !std::path::Path::new(&album_path).exists() || !db::check_album_exist_by_id(&client, &album_id).await {
        error!("Error occured : album with id={} not found on disk", &album_id);
        return Err(HandlerError::InternalError);
    }
    // Check user has right to write: No need to do here because admin path
    
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