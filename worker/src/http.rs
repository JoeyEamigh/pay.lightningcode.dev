use axum::{
  extract::{self, rejection::JsonRejection},
  http::status::StatusCode,
  response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(extract::FromRequest)]
#[from_request(via(axum::Json), rejection(PayError))]
pub struct PayJson<T>(pub T);

impl<T> IntoResponse for PayJson<T>
where
  axum::Json<T>: IntoResponse,
{
  fn into_response(self) -> Response {
    axum::Json(self.0).into_response()
  }
}

#[derive(Serialize)]
struct PayErrorResponse {
  code: u16,
  message: String,
}

pub enum PayError {
  NotFound(String),
  DeserializeError(String),
  BadRequest(String),
  InternalError(String),
  JsonRejection(JsonRejection),
}

impl From<reqwest::Error> for PayError {
  fn from(err: reqwest::Error) -> Self {
    Self::InternalError(format!("reqwest error: {:?}", err))
  }
}

impl IntoResponse for PayError {
  fn into_response(self) -> Response {
    let (status, message) = match self {
      PayError::NotFound(message) => (StatusCode::NOT_FOUND, message),
      PayError::DeserializeError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
      PayError::InternalError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
      PayError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
      PayError::JsonRejection(rejection) => (rejection.status(), rejection.body_text()),
    };

    (
      status,
      PayJson(PayErrorResponse {
        code: status.into(),
        message,
      }),
    )
      .into_response()
  }
}

impl From<JsonRejection> for PayError {
  fn from(rejection: JsonRejection) -> Self {
    Self::JsonRejection(rejection)
  }
}
