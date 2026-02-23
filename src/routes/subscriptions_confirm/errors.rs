use crate::domain::SubscriptionTokenError;
use actix_web::error::ResponseError;
use actix_web::http::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum ConfirmError {
    #[error("Invalid subscription token: {0:?}")]
    InvalidToken(#[from] SubscriptionTokenError),
    #[error("Subscription token not found")]
    TokenNotFound,
    #[error("Database error: {0:?}")]
    DatabaseError(#[from] sqlx::Error),
}

impl ResponseError for ConfirmError {
    fn status_code(&self) -> StatusCode {
        match self {
            ConfirmError::InvalidToken(_) => StatusCode::BAD_REQUEST,
            ConfirmError::TokenNotFound => StatusCode::UNAUTHORIZED,
            ConfirmError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
