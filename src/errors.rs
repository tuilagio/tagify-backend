use actix_web::{
    HttpResponse,
    ResponseError,
    http::header,
};
use failure::Fail;

use actix_web::http::StatusCode;
use actix_http::ResponseBuilder;

#[derive(Fail, Debug)]
pub enum UserError {
    #[fail(display = "Parsing error on field: {}", field)]
    BadClientData { field: String  },
    #[fail(display = "An internal error occured. Try again later")]
    InternalError
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
            UserError::BadClientData {..} => StatusCode::BAD_REQUEST,
        }
    }
}