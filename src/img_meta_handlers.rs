use crate::models::{ Hash, ReceivedUser, User, InternalUser, ReceivedLoginData, ReceivedAlbumMeta, InternalAlbumMeta};
use crate::errors::{UserError, CutomResponseError};
use crate::utils;
use crate::db;
use crate::errors;

use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use actix_web::http::StatusCode;
use deadpool_postgres::Pool;
use log::{debug, error};

use serde_json::{json};

/// Procedure to work with image
/// An image has a meta. Meta holds meta data (date time, filename, id, ...) and tag list.
/// - Create: POST meta to receive img_id for image. Then send a POST request to images/<img_id> to upload.
/// - Re-create image: send PUT to images/<img_id>. server will silently replace old img.
/// - Lock: The moment a tagger clicks mouse on tag input form, the lock request is sent. Server remember who created the lock and when. 
///         If there is no lock for this img he will receive a 200-accept from server. His frontend can implement a countdown or not, just to inform him. 
///            He can also send a unlock request to tell server he doesn't want to submit anything yet to delete his lock.
///            While holding the lock the tagger can also send lock request again to extend. Think about Berlin Wohnung Mieter!
///         Else, 423-Locked. He better waits!
///         Any PUT or POST request to upload tags will be check: 
///         + if timeout from last lock for this img still lasts => only request from this tagger gets accept, other gets 403-Forbidden. 
///           Server deletes this lock after applying his submit.
///         + else if timeout past (or no lock at all) ANY request will be accepted (without sending lock request first) and lock gets deleted.



pub async fn create_meta(
    pool: web::Data<Pool>,
    data: web::Path<(String,)>,
    // data_payload: web::Json<ReceivedImageMeta>,
) -> Result<HttpResponse, CutomResponseError> {

    println!("{:?}", data);
    return Err(CutomResponseError::NotImplemented {
        message: "'create_meta' not implemented yet. Remove after done.".to_string(),
    });
}

pub async fn delete_meta(
    pool: web::Data<Pool>,
    data: web::Path<(String, String)>,
) -> Result<HttpResponse, CutomResponseError> {

    println!("{:?}", data);
    return Err(CutomResponseError::NotImplemented {
        message: "'delete_meta' not implemented yet. Remove after done.".to_string(),
    });
}

pub async fn delete_all_metas(
    pool: web::Data<Pool>,
    data: web::Path<(String,)>,
) -> Result<HttpResponse, CutomResponseError> {

    return Err(CutomResponseError::NotImplemented {
        message: "'delete_all_metas' not implemented yet. Remove after done.".to_string(),
    });
}

pub async fn get_metas(
    req: HttpRequest,
    pool: web::Data<Pool>,
    data: web::Path<(String,)>,
) -> Result<HttpResponse, CutomResponseError> {

    let temp: String = req.query_string().to_string(); // status=tagged&test=true
    println!("status {} data {:?}", temp, data);
    
    // TODO: complete this implementation
    /* 
    let query = match utils::query_string_to_queries(&req.query_string().to_string()) {
        Ok(qry) => {
            if qry.len() == 0 || qry.get("status") == None {
                // TODO: return all image metas of this album
            }
            match qry.get("status") {
                Some(status) => {
                    if status == "untagged" {
                        // TODO: return all untagged image metas
                    } else if status == "tagged" {
                        // TODO: return alltagged image metas
                    } else {
                        return Err(CutomResponseError::BadClientDataParse {
                            field: "'status' should only be 'untagged' or 'tagged'".to_string(),
                        })
                    }
                },
                None => return Err(CutomResponseError::InternalError)
            }
        },
        Err(e) => return Err(CutomResponseError::InternalError)
    };
    */
    return Err(CutomResponseError::NotImplemented {
        message: "'get_metas' not implemented yet. Remove after done.".to_string(),
    });
}

pub async fn get_meta(
    pool: web::Data<Pool>,
    data: web::Path<(String, String)>,
) -> Result<HttpResponse, CutomResponseError> {

    return Err(CutomResponseError::NotImplemented {
        message: "'get_meta' not implemented yet. Remove after done.".to_string(),
    });
}

pub async fn update_meta(
    pool: web::Data<Pool>,
    data: web::Path<(String, String)>,
    // data_payload: web::Json<ReceivedImageMeta>,
) -> Result<HttpResponse, CutomResponseError> {

    return Err(CutomResponseError::NotImplemented {
        message: "'update_meta' not implemented yet. Remove after done.".to_string(),
    });
}