use crate::models::{ Hash, ReceivedUser, User, InternalUser, ReceivedLoginData, ROLES};
use crate::errors::{UserError, HandlerError};
use crate::utils;
use crate::db;
use crate::errors;

use actix_identity::Identity;
use actix_web::{web, HttpResponse, Result};
use actix_web::http::StatusCode;
use deadpool_postgres::Pool;
use log::{debug, error};

use serde_json::{json};

pub async fn logout(id: Identity) -> Result<HttpResponse, UserError> {
    // Check if logged in
    if let None = id.identity() {
        return Err(UserError::AuthFail);
    }
    id.forget();
    Ok(HttpResponse::new(StatusCode::OK))
}

pub async fn login(
    data: web::Json<ReceivedLoginData>,
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

    let user: InternalUser = match db::get_internal_user(&client, &data.username).await {
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

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(json!({
            "id": user.id,
            "username": user.username,
            "nickname": user.nickname,
            "role": user.role,
        }))
    )
}

pub async fn whoami(
    pool: web::Data<Pool>,
) -> Result<HttpResponse, HandlerError> {
    // let client = match pool.get().await {
    //     Ok(item) => item,
    //     Err(e) => {
    //         error!("Error occured : {}", e);
    //         return Err(HandlerError::InternalError);
    //     }
    // };

    /* Get user id from cookies or whatever we used */
    // TODO: implement this. Harded code "2"
    // let user_id: i32 = 2;

    return Err(HandlerError::NotImplemented {
        message: "Not implemented yet. Remove after done.".to_string(),
    });
    /* 
    let user = db::get_user_postgres_with_id(client, &user_id).await;
    match user {
        Ok(u) => Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(json!({
                "id": u.username,
                "username": u.id,
                "nickname": u.nickname,
                "role": u.role,
            }))
        ),
        Err(e) => {
            println!("{:?}", e);
            return Err(HandlerError::InternalError)
        },
    }
    */
}

pub async fn delete_user(
    pool: web::Data<Pool>,
    data: web::Path<(String,)>,
) -> Result<HttpResponse, HandlerError> {

    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    debug!("User delete debug: {}", data.0);

    // TODO: Need to be an admin to perform DELETE user. Wait for AuthAdmin middleware

    /* Get user id from url parameter */
    // Check if id numberic
    if !utils::is_string_numeric(data.0.clone()) {
        return Err(HandlerError::BadClientDataParse {
            field: "User id should be numeric".to_string(),
        });
    }
    let user_id: i32 = data.0.parse().unwrap();
    
    /* Update to db */
    let result = db::delete_user_with_id(&client, &user_id).await;
    match result {
        Ok(return_user) => Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(json!({
                "id": return_user.id,
                "username": return_user.username,
                "nickname": return_user.nickname,
                "role": return_user.role,
            }))
        ),
        Err(e) => {
            println!("{:?}", e);
            return Err(HandlerError::InternalError)
        },
    }
}

pub async fn create_user(
    pool: web::Data<Pool>,
    data: web::Json<ReceivedUser>,
) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    debug!("{:#?}", data);

    //Check if password is not empty
    if data.password.len() == 0 {
        return Err(HandlerError::BadClientDataParse {
            field: "password field cannot be empty".to_string(),
        });
    }

    if data.username.len() == 0 {
        return Err(HandlerError::BadClientDataParse {
            field: "username field cannot be empty".to_string(),
        });
    }
    
    // TODO: check if username existed

    if data.role.len() == 0 {
        return Err(HandlerError::BadClientDataParse {
            field: "username field cannot be empty".to_string(),
        });
    } else if !(ROLES.iter().any(|&i| i==data.role)) {
        return Err(HandlerError::BadClientDataParse {
            field: "User role unvalid".to_string(),
        });
    }

    let mut user = User::create_user(&data.username, &data.nickname, &data.password, &data.role);

    if let Err(e) = user.hash_password() {
        error!("Error occured: {}", e);
        return Err(HandlerError::InternalError);
    }

    /* Update to db */
    let result = db::create_user(&client, &user).await;
    match result {
        Ok(return_user) => Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(json!({
                "id": return_user.id,
                "username": return_user.username,
                "nickname": return_user.nickname,
                "role": return_user.role,
            }))
        ),
        Err(e) => {
            println!("{:?}", e);
            return Err(HandlerError::InternalError)
        },
    }
}

pub async fn get_user(
    pool: web::Data<Pool>,
    data: web::Path<(String,)>,
) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    /* Get user id from url parameter */
    // Check if id numberic
    if !utils::is_string_numeric(data.0.clone()) {
        return Err(HandlerError::BadClientDataParse {
            field: "User id should be numeric".to_string(),
        });
    }
    let user_id: i32 = data.0.parse().unwrap();

    let user = db::get_internal_user_with_id(&client, &user_id).await;
    match user {
        Ok(u) => Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(json!({
                "id": u.id,
                "username": u.username,
                "nickname": u.nickname,
                "role": u.role,
            }))
        ),
        Err(e) => {
            println!("{:?}", e);
            return Err(HandlerError::InternalError)
        },

    }
}

pub async fn update_user (
    pool: web::Data<Pool>,
    data: web::Path<(String,)>,
    data_payload: web::Json<ReceivedUser>,
) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    /* Get user id from url parameter */
    // Check if id numberic
    if !utils::is_string_numeric(data.0.clone()) {
        return Err(HandlerError::BadClientDataParse {
            field: "User id should be numeric".to_string(),
        });
    }
    let user_id: i32 = data.0.parse().unwrap();

    /* Check if original user exists first */
    let old_user = db::get_internal_user_with_id(&client, &user_id).await;
    match old_user {
        Ok(o_u) => {
            //Check if password is not empty
            if data_payload.password.len() == 0 {
                return Err(HandlerError::BadClientDataParse {
                    field: "password field cannot be empty".to_string(),
                });
            }

            if data_payload.username != o_u.username{
                return Err(HandlerError::BadClientDataParse {
                    field: "username can't be change. Sorry".to_string(),
                });
            }
            
            if data_payload.role.len() == 0 {
                return Err(HandlerError::BadClientDataParse {
                    field: "username field cannot be empty".to_string(),
                });
            } else if !(ROLES.iter().any(|&i| i==data_payload.role)) {
                return Err(HandlerError::BadClientDataParse {
                    field: "User role unvalid".to_string(),
                });
            }

            let mut user = User::create_user(
                &o_u.username, &data_payload.nickname, 
                &data_payload.password, &data_payload.role
            );

            if let Err(e) = user.hash_password() {
                error!("Error occured : {}", e);
                return Err(HandlerError::InternalError);
            }
            
            /* Everything find. Update to db */
            let result = db::update_user_with_id(&client, &o_u.id, &user).await;
            match result {
                Ok(return_user) => Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(json!({
                    "id": return_user.id,
                    "username": return_user.username,
                    "nickname": return_user.nickname,
                    "role": return_user.role,
                }))
                ),
                Err(e) => {
                    println!("{:?}", e);
                    return Err(HandlerError::InternalError)
                },
            }
        },
        Err(e) => {
            println!("{:?}", e);
            return Err(HandlerError::InternalError);
        },

    }
}

pub async fn get_user_albums(
    pool: web::Data<Pool>,
    data: web::Path<(String,)>,
) -> Result<HttpResponse, HandlerError> {
    // let client = match pool.get().await {
    //     Ok(item) => item,
    //     Err(e) => {
    //         error!("Error occured : {}", e);
    //         return Err(HandlerError::InternalError);
    //     }
    // };
    
    /* Get user id from url parameter */
    // Check if id numberic
    // if !utils::is_string_numeric(data.0.clone()) {
    //     return Err(HandlerError::BadClientDataParse {
    //         field: "User id should be numeric".to_string(),
    //     });
    // }
    // let user_id: i32 = data.0.parse().unwrap();

    // TODO:
    // - Create album model
    // - Implement db::get_owned_albums_for_user_id()
    // - Implement db::get_shared_albums_for_user_id()
    // - Construct return response as documented
    /* 
    let owned_albums = db::get_owned_albums_for_user_id(client, &user_id).await;
    let shared_albums = db::get_shared_albums_for_user_id(client, &user_id).await;
    match owned_albums {
        Ok(u) => Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(json!({
                "id": user_id,
                "owned": [<owned_go_here>],
                "owned": [<owned_go_here>],
            }))
        ),
        Err(e) => {
            println!("{:?}", e);
            return Err(HandlerError::InternalError)
        },
    }
    */

    return Err(HandlerError::NotImplemented {
        message: "Not implemented yet. Remove after done.".to_string(),
    });
}
