use crate::album_models::CreateAlbum;
use crate::errors::HandlerError;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Result};
use deadpool_postgres::Pool;
use log::error;

use crate::db;

pub async fn create_album(
    pool: web::Data<Pool>,
    data: web::Json<CreateAlbum>,
) -> Result<HttpResponse, HandlerError> {
    let album = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let result = match db::create_album(&album, &data).await {
        Err(e) => {
            error!("Error occured after create_album: {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(item) => item,
    };

    //TODO create album folder on photo_server

    Ok(HttpResponse::build(StatusCode::OK).json(result))
}
