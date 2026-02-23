use crate::domain::NewSubscriberError;
use actix_web::error::ResponseError;
use actix_web::http::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum SubscribeError {
    #[error("Invalid new subscriber input: {0:?}")]
    ValidationError(#[from] NewSubscriberError),
    #[error("Database error: {0:?}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Failed to send confirmation email: {0:?}")]
    ConfirmationEmailError(#[from] reqwest::Error),
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscribeError::DatabaseError(_) | SubscribeError::ConfirmationEmailError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}
