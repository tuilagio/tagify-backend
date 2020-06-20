use crate::album_models::{CreateUser, UpdateUserAdmin, User};
use crate::errors::HandlerError;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Result};
use deadpool_postgres::Pool;
use log::error;

use crate::db;

pub async fn create_album(
    pool: web::Data<Pool>,
    data: web::Json<CreateUser>,
) -> Result<HttpResponse, HandlerError> {
    let album = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    let result = match db::create_album(&client, &data).await {
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(item) => item,
    };

    Ok(HttpResponse::build(StatusCode::OK).json(result))
}
