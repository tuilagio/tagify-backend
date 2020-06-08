use crate::errors::UserError;
use crate::models::User;
use crate::models::ReceivedUser;
use actix_web::http::StatusCode;
use actix_web::{delete, put, web, HttpResponse, Result};
use deadpool_postgres::Pool;
use log::{debug, error};

#[put("/create_admin")]
async fn create_admin(
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

#[delete("/delete_admin/{username}")]
async fn delete_admin(
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
