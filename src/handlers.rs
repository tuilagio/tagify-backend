use crate::errors::UserError;
use crate::models::{ SendUser,Status, User, Nickname, Password, Hash, ReceivedLoginData};
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Result, Responder, HttpRequest};
use deadpool_postgres::Pool;
use log::{debug, error};

use crate::my_identity_service::{Identity, login_user};
use crate::db;
use crate::errors;

pub async fn status() -> impl Responder {
    web::HttpResponse::Ok().json(Status {
        status: "server is working :D".to_string(),
    })
}


pub async fn get_user(pool: web::Data<Pool>, id: Identity) -> Result<HttpResponse, UserError> {
    // // Check if logged in
    let user= id.identity();

    let send_user = SendUser {
        username: user.username,
        nickname: user.nickname,
        is_admin: user.is_admin,
    };

    Ok(HttpResponse::build(StatusCode::OK).json(send_user))
}

pub async fn logout(id: Identity) -> Result<HttpResponse, UserError> {
    // Check if logged in

    id.forget();

    Ok(HttpResponse::new(StatusCode::OK))
}

pub async fn login(
    data: web::Json<ReceivedLoginData>,
    pool: web::Data<Pool>,
    req: HttpRequest
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
    login_user(req, user);
    // id.remember(user);

    Ok(HttpResponse::new(StatusCode::OK))
}


pub async fn update_nickname(pool: web::Data<Pool>, id: Identity, data: web::Json<Nickname>) -> Result<HttpResponse, UserError> {

    let user = id.identity();
    // Check if logged in
    // let username = match id.identity() {
    //     Some(id) => id,
    //     None => return Err(UserError::AuthFail)
    // };

    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(UserError::InternalError);
        }
    };

    // Check if nickname is not empty & exists
    let nickname = match &data.nickname {
        Some(item) => if item.len() == 0 {
            return Err(UserError::BadClientData{field: "nickname cannot be empty".to_string()  });
        }
        else {
            item
        },
        None => {
            return Err(UserError::BadClientData{field: "couldn't find nickname".to_string()  });
        }
    };


    let result = client.execute("UPDATE users SET nickname = $1 WHERE username = $2", &[&nickname,&user.username]).await;

    match result {
        Err(e) => {
            error!("Error occured: {}",e );
            return Err(UserError::InternalError);
        },
        Ok(num_updated) => num_updated
    };


    Ok(HttpResponse::new(StatusCode::OK))
}


pub async fn update_password(pool: web::Data<Pool>, id: Identity, mut data: web::Json<Password>) -> Result<HttpResponse, UserError> {

    // Check if logged in
    let user = id.identity();


    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(UserError::InternalError);
        }
    };

    //Check if password is not empty
    if data.password.len() == 0 {
        return Err(UserError::BadClientData{field: "password cannot be empty".to_string()});
    }

    // Check if password and repeatPassword match
    if !data.password.eq(&data.repeat_password){
        return Err(UserError::BadClientData{field: "password and password repeat don't match".to_string()});
    }

    // hash password
    if let Err(e) = data.hash_password() {
        error!("Error occured: {}", e);
        return Err(UserError::InternalError);
    }

    //execute db query
    let result = client.execute("UPDATE users SET password = $1 WHERE username = $2", &[&data.password,&user.username]).await;

    match result {
        Err(e) => {
            error!("Error occured: {}",e );
            return Err(UserError::InternalError);
        },
        Ok(num_updated) => num_updated
    };


    Ok(HttpResponse::new(StatusCode::OK))
}
