pub mod apiparam;

use actix_web::{
    error,
    web,
    Result,
    http:: {
        header::ContentType,
        StatusCode,
    },
    App,
    guard::{
        Guard,
        GuardContext,
    },
    HttpResponse,
};
use derive_more::{Display, Error};
use serde::Serialize;

#[derive(Debug, Display, Serialize, Error)]
#[display(fmt = "api error: {:?}", message)]
pub struct ApiError {
    pub message: String,
}

impl error::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(serde_json::to_string(&self).unwrap())
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

struct ApiAuth;

impl Guard for ApiAuth {
    fn check(&self, ctx: &GuardContext<'_>) -> bool {
        let token = ctx.head().headers().get("token");
        if let Some(t) = token {
            return t == "token"
        }
        false
    }
}