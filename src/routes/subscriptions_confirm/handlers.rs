use super::ConfirmError;
use super::helpers::*;

use crate::domain::{SubscriptionToken, SubscriptionTokenError};

use actix_web::{HttpResponse, web};
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

impl TryFrom<Parameters> for SubscriptionToken {
    type Error = SubscriptionTokenError;

    fn try_from(value: Parameters) -> Result<Self, Self::Error> {
        SubscriptionToken::parse(value.subscription_token)
    }
}

#[tracing::instrument(
    name = "Confirm a pending subscriber"
    skip(parameters, pool)
)]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ConfirmError> {
    let subscription_token: SubscriptionToken = parameters.0.try_into()?;

    match get_subscriber_id_from_token(&pool, &subscription_token).await? {
        Some(subscriber_id) => {
            confirm_subscriber(&pool, subscriber_id).await?;
            Ok(HttpResponse::Ok().finish())
        }
        None => Err(ConfirmError::TokenNotFound),
    }
}
