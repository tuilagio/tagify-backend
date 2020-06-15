use actix_web::{http::header, HttpResponse, ResponseError};
use failure::Fail;

use actix_http::ResponseBuilder;
use actix_web::http::StatusCode;
use serde::Serialize;
use serde_json::{json, to_string_pretty};
/*
 * Only to be used in admin_handlers.rs & handlers.rs
 */
#[derive(Fail, Debug)]
pub enum UserError {
    #[fail(display = "Parsing error on field: {}", field)]
    BadClientDataParse { field: String },
    #[fail(display = "An internal error occured. Try again later")]
    InternalError,
    #[fail(display = "You are not logged in")]
    AuthFail,
}

impl ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        ResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            UserError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::BadClientDataParse { .. } => StatusCode::BAD_REQUEST,
            UserError::AuthFail => StatusCode::UNAUTHORIZED,
        }
    }
}

#[derive(Fail, Debug)]
pub enum HandlerError {
    #[fail(display = "Parsing error on field: {}", field)]
    BadClientDataParse { field: String },
    #[fail(display = "{}", err)]
    BadClientData { err: String },
    #[fail(display = "An internal error occured. Try again later")]
    InternalError,
    #[fail(display = "You are not logged in")]
    AuthFail,
    #[fail(display = "{}", message)]
    NotImplemented { message: String },
}

/* Hin: self.to_string() is the text in fail(display ....) */
impl ResponseError for HandlerError {
    fn error_response(&self) -> HttpResponse {
        ResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .body(json!({"message": self.to_string()}))
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            HandlerError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            HandlerError::BadClientDataParse { .. } => StatusCode::BAD_REQUEST,
            HandlerError::BadClientData { .. } => StatusCode::BAD_REQUEST,
            HandlerError::AuthFail => StatusCode::UNAUTHORIZED,
            HandlerError::NotImplemented { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/*
 * Only to be used in db.rs
 */
#[derive(Fail, Debug)]
pub enum DBError {
    #[fail(display = "Query to struct mapper error")]
    MapperError(tokio_pg_mapper::Error),

    #[fail(display = "Postgres error")]
    PostgresError(tokio_postgres::Error),
}
impl From<tokio_postgres::Error> for DBError {
    fn from(err: tokio_postgres::Error) -> DBError {
        DBError::PostgresError(err)
    }
}
impl From<tokio_pg_mapper::Error> for DBError {
    fn from(err: tokio_pg_mapper::Error) -> DBError {
        DBError::MapperError(err)
    }
}

