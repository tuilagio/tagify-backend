use crate::db;
use crate::errors::UserError;
use crate::models::Todo;

use actix_files::NamedFile;
use actix_web::http::StatusCode;
use actix_web::{delete, get, put, web, HttpResponse, Responder, Result};
use deadpool_postgres::{Client, Pool};
use log::{debug, error};
use std::path::PathBuf;

#[get("/")]
async fn index() -> Result<NamedFile> {
    let path: PathBuf = PathBuf::from("../debug_dist/index.html");
    Ok(NamedFile::open(path)?)
}

//maybe better version
#[get("/get_all_todos")]
async fn get_all_todos(pool: web::Data<Pool>) -> impl Responder {
    let client: Client = pool.get().await.expect("Error geting todo's from DB");
    let result = db::get_all_todos_db(&client).await;
    match result {
        Ok(todos) => HttpResponse::Ok().json(todos),
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}

#[put("/add_todo")]
async fn add_todo(pool: web::Data<Pool>, data: web::Json<Todo>) -> Result<HttpResponse, UserError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured : {}", e);
            return Err(UserError::InternalError);
        }
    };

    debug!("{:#?}", data);

    // Query data
    let result = client
        .execute(
            "INSERT INTO todos (description, date, progress) VALUES ($1,$2,$3)",
            &[&data.description, &data.date, &data.progress],
        )
        .await;

    if let Err(e) = result {
        error!("Error occured: {}", e);
        return Err(UserError::InternalError);
    }

    Ok(HttpResponse::new(StatusCode::OK))
}

#[delete("/delete_todo/{id}")]
async fn delete_todo(
    pool: web::Data<Pool>,
    data: web::Path<(i32,)>,
) -> Result<HttpResponse, UserError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(UserError::InternalError);
        }
    };

    debug!("Todo id tos delete: {}", data.0);

    // Query data
    let result = client
        .execute("DELETE FROM todos WHERE id = $1", &[&data.0])
        .await;

    match result {
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(UserError::InternalError);
        }
        Ok(num_updated) => {
            if num_updated == 0 {
                return Err(UserError::BadClientData {
                    field: "id does not exist".to_string(),
                });
            }
        }
    };

    Ok(HttpResponse::new(StatusCode::OK))
}

#[put("/update_todo")]
async fn update_todo(
    pool: web::Data<Pool>,
    data: web::Json<Todo>,
) -> Result<HttpResponse, UserError> {
    let client = match pool.get().await {
        Ok(item) => item,
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(UserError::InternalError);
        }
    };

    debug!("{:#?}", data);

    let id = match data.id {
        Some(i) => i,
        None => {
            return Err(UserError::BadClientData {
                field: "id is missing".to_string(),
            })
        }
    };

    // Query data
    let result = client
        .execute(
            "UPDATE todos SET description = $1, date = $2, progress = $3 WHERE id = $4",
            &[&data.description, &data.date, &data.progress, &id],
        )
        .await;

    match result {
        Err(e) => {
            error!("Error occured: {}", e);
            return Err(UserError::InternalError);
        }
        Ok(num_updated) => {
            if num_updated == 0 {
                return Err(UserError::BadClientData {
                    field: "id does not exist".to_string(),
                });
            }
        }
    };

    Ok(HttpResponse::new(StatusCode::OK))
}
