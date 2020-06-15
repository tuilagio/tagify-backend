use crate::models::{ Hash, ReceivedUser, User, InternalUser, ReceivedLoginData, ReceivedAlbumMeta, InternalAlbumMeta};
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

pub async fn delete_album_meta(
    pool: web::Data<Pool>,
    data: web::Path<(String,)>,
) -> Result<HttpResponse, HandlerError> {
    /* 
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
    };
    // TODO: check delete permission

    /* Check if id numberic */
    if !utils::is_string_numeric(data.0.clone()) {
        return Err(HandlerError::BadClientDataParse {
            field: "Album id should be numeric".to_string(),
        });
    }
    let album_id: i32 = data.0.parse().unwrap();

    // TODO: implement db query stuff in "db.rs" and call here
    let result = db::delete_album_meta_with_id(&client, &album_id).await;

    match result {
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
        Ok(num_updated) => {
            if num_updated == 0 {
                return Err(HandlerError::BadClientData {
                    // TODO: consider hide sensitive error information to avoid data exploiting
                    err: "Album does not exist".to_string(),
                });
            }
        }
    };

    Ok(HttpResponse::new(StatusCode::OK))
    */

    return Err(HandlerError::NotImplemented {
        message: "'delete_album_meta' not implemented yet. Remove after done.".to_string(),
    });
}

pub async fn create_album_meta(
    pool: web::Data<Pool>,
    data: web::Json<ReceivedAlbumMeta>,
) -> Result<HttpResponse, HandlerError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(HandlerError::InternalError);
        }
    };

    // TODO: check data correctness: album exists, ...

    // TODO: hide SQL interaction in db.rs and call it here

    // TODO: return created user in response. Use client.query_one(".... RETURNING *") 
    // in db.rs implementation to get inserted row back
    /* {
        "date_created": "2016-06-22 19:10:25-07",
        "last_modified": "2016-06-22 19:10:25-07",
        "id": 1,
        "name": "Dog cat portrait",
        "tags": ["dog", "cat", "undefined"],
        "description": "This is an album of (hopefully) dog and cat images. Tag it my dudes!",
        "image_number": 999,
        "tagged_number": 0,
        "owner": {
          "nickname": "user2",
          "id": 2},
        "taggers": [
          {"nickname": "tagger3", "id": 3},
          {"nickname": "tagger4", "id": 4}
        ],
        "thumbnail": ""
      } */

    return Err(HandlerError::NotImplemented {
        message: "'create_album_meta' not implemented yet. Remove after done.".to_string(),
    });
}

pub async fn get_album_meta(
    pool: web::Data<Pool>,
    data: web::Path<(String,)>,
) -> Result<HttpResponse, HandlerError> {
    /* 
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
    };
    // TODO: check delete permission

    /* Check if id numberic */
    if !utils::is_string_numeric(data.0.clone()) {
        return Err(HandlerError::BadClientDataParse {
            field: "Album id should be numeric".to_string(),
        });
    }
    let album_id: i32 = data.0.parse().unwrap();

    // TODO: implement db query stuff in "db.rs" and call here
    let album = db::get_album_meta_with_id(&client, &album_id).await;

    match album {
        Ok(a) => Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(json!({
                "id": a.id,
                // TODO: construct response json
            }))
        ),
        Err(e) => {
            println!("{:?}", e);
            return Err(HandlerError::InternalError)
        },

    }
    */

    return Err(HandlerError::NotImplemented {
        message: "'get_album_meta' not implemented yet. Remove after done.".to_string(),
    });
}

pub async fn update_album_meta (
    pool: web::Data<Pool>,
    data: web::Path<(String,)>,
    data_payload: web::Json<ReceivedAlbumMeta>,
) -> Result<HttpResponse, HandlerError> {
    /* 
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(HandlerError::InternalError);
        }
    };
    // TODO: check update permission

    /* Check if id numberic */
    if !utils::is_string_numeric(data.0.clone()) {
        return Err(HandlerError::BadClientDataParse {
            field: "Album id should be numeric".to_string(),
        });
    }
    let album_id: i32 = data.0.parse().unwrap();

    // TODO: check if album with this id exists
    let old_album = db::get_album_meta_with_id(&client, &album_id).await;
    match old_album {
        Ok(o_a) => {
            // TODO: implement db query stuff in "db.rs" and call here
            let album = db::update_album_meta_with_id(&client, &album_id).await;

            match album {
                Ok(a) => Ok(HttpResponse::Ok()
                    .content_type("application/json")
                    .body(json!({
                        "id": a.id,
                        // TODO: construct response json
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
            return Err(HandlerError::InternalError)
        },
    }
    */

    return Err(HandlerError::NotImplemented {
        message: "'update_album_meta' not implemented yet. Remove after done.".to_string(),
    });
}

