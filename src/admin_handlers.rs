use crate::errors::UserError;
use crate::models::{ReceivedUser, User, Hash};
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Result};
use deadpool_postgres::Pool;
use log::{debug, error};

pub async fn create_admin(
    pool: web::Data<Pool>,
    data: web::Json<ReceivedUser>,
) -> Result<HttpResponse, UserError> {

    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(UserError::InternalError);
        }
    };

    debug!("{:#?}", data);

    //Check if password is not empty
    if data.password.len() == 0 {
        return Err(UserError::BadClientData{field: "password field cannot be empty".to_string()});
    }

    if data.username.len() == 0 {
        return Err(UserError::BadClientData{field: "username field cannot be empty".to_string()});
    }

    // Check if password and repeatPassword match
    if !data.password.eq(&data.repeat_password){
        return Err(UserError::BadClientData{field: "password and password repeat fields don't match".to_string()});
    }

    let mut admin = User::create_user(&data.username, &data.password, true);

    if let Err(e) = admin.hash_password() {
        error!("Error occured: {}", e);
        return Err(UserError::InternalError);
    }

    // Query data
    let result = client
        .execute(
            "INSERT INTO users (username, nickname, password, is_admin) VALUES ($1,$2,$3,$4)",
            &[
                &admin.username,
                &admin.nickname,
                &admin.password,
                &admin.is_admin,
            ],
        )
        .await;

    if let Err(e) = result {
        error!("Error occured: {}", e);
        return Err(UserError::InternalError);
    }

    Ok(HttpResponse::new(StatusCode::OK))
}

pub async fn delete_admin(
    pool: web::Data<Pool>,
    data: web::Path<(String,)>,
) -> Result<HttpResponse, UserError> {

    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(UserError::InternalError);
        }
    };

    debug!("Admin delete debug: {}", data.0);

    // Query data
    let result = client
        .execute("DELETE FROM users WHERE username = $1", &[&data.0])
        .await;

    match result {
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(UserError::InternalError);
        }
        Ok(num_updated) => {
            if num_updated == 0 {
                return Err(UserError::BadClientData {
                    field: "user does not exist".to_string(),
                });
            }
        }
    };

    Ok(HttpResponse::new(StatusCode::OK))
}
