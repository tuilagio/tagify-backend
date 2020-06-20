use crate::album_models::CreateAlbum;
use crate::user_models::User;

use crate::errors::HandlerError;
use crate::my_identity_service::Identity;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Result};
use deadpool_postgres::Pool;
use log::error;

use crate::db;

pub async fn create_album(
    pool: web::Data<Pool>,
    data: web::Json<CreateAlbum>,
    id: Identity,
) -> Result<HttpResponse, HandlerError> {
    let user: User = id.identity();
    let first_photo = String::from("default_path");

    let album = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
    };
    //create album without tags
    let result = match db::create_album(&album, &data, user.id, first_photo).await {
        Err(e) => {
            error!("Error occured after create_album: {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(item) => item,
    };
    //TODO create album folder on photo_server

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
