use crate::errors::UserError;
use crate::models::{ReceivedUser, SendUser, Status, User};
use actix_identity::Identity;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Result, Responder};
use deadpool_postgres::Pool;
use log::{debug, error};

use crate::db;
use crate::errors;

pub async fn status() -> impl Responder {
    web::HttpResponse::Ok().json(Status {
        status: "server is working :D".to_string(),
    })
}

pub async fn get_user(pool: web::Data<Pool>, id: Identity) -> Result<HttpResponse, UserError> {
    // Check if logged in
    let username = match id.identity() {
        Some(id) => id,
        None => return Err(UserError::AuthFail),
    };

    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(UserError::InternalError);
        }
    };
    let user: User = match db::get_user(client, &username).await {
        Ok(user) => user,
        Err(e) => match e {
            errors::DBError::PostgresError(e) => {
                error!("Getting user failed: {}", e);
                return Err(UserError::AuthFail);
            }
            errors::DBError::MapperError(e) => {
                error!("Error occured: {}", e);
                return Err(UserError::InternalError);
            }
        },
    };

    let send_user = SendUser {
        username: user.username,
        nickname: user.nickname,
        is_admin: user.is_admin,
    };

    Ok(HttpResponse::build(StatusCode::OK).json(send_user))
}

pub async fn logout(id: Identity) -> Result<HttpResponse, UserError> {
    // Check if logged in
    if let None = id.identity() {
        return Err(UserError::AuthFail);
    }

    id.forget();

    Ok(HttpResponse::new(StatusCode::OK))
}

pub async fn login(
    data: web::Json<ReceivedUser>,
    pool: web::Data<Pool>,
    id: Identity,
) -> Result<HttpResponse, UserError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(UserError::InternalError);
        }
    };

    let user: User = match db::get_user(client, &data.username).await {
        Ok(user) => user,
        Err(e) => match e {
            errors::DBError::PostgresError(e) => {
                error!("Getting user failed: {}", e);
                return Err(UserError::AuthFail);
            }
            errors::DBError::MapperError(e) => {
                error!("Error occured: {}", e);
                return Err(UserError::InternalError);
            }
        },
    };

    match user.verify_password(data.password.as_bytes()) {
        Ok(correct) => {
            if !correct {
                return Err(UserError::AuthFail);
            }
        }
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(UserError::InternalError);
        }
    }

    debug!("User {} logged in successfully", user.username);
    id.remember(user.username.to_owned());

    Ok(HttpResponse::new(StatusCode::OK))
}
