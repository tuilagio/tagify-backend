use crate::album_models::{CreateAlbum, AlbumsPreview };
use crate::user_models::{User};

use crate::errors::{HandlerError, DBError};
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
    //add all tags to db
    //TODO add tags

    //TODO connect tags with ablum

    //TODO create album folder on photo_server

    Ok(HttpResponse::build(StatusCode::OK).json(result))
}

// gets all albums data (id, title, description, first_photo)
pub async fn get_all_albums(
    pool: web::Data<Pool>,
    id: Identity,
) -> Result<HttpResponse, HandlerError> {
    
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
                error!("Getting user failed: {}", e);
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