use crate::domain::SubscriptionToken;

use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

impl TryFrom<Parameters> for SubscriptionToken {
    type Error = String;

    fn try_from(value: Parameters) -> Result<Self, Self::Error> {
        SubscriptionToken::parse(value.subscription_token)
    }
}

#[tracing::instrument(
    name = "Confirm a pending subscriber"
    skip(parameters, pool)
)]
pub async fn confirm(parameters: web::Query<Parameters>, pool: web::Data<PgPool>) -> HttpResponse {
    let subscription_token: SubscriptionToken = match parameters.0.try_into() {
        Ok(subscription_token) => subscription_token,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let id = match get_subscriber_id_from_token(&pool, &subscription_token).await {
        Ok(id) => id,
        Err(err) => {
            tracing::error!("Getting subscriber id failed: {:?}", err);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match id {
        Some(subscriber_id) => {
            if let Err(err) = confirm_subscriber(&pool, subscriber_id).await {
                tracing::error!("Confirm subscriber failed: {:?}", err);
                return HttpResponse::InternalServerError().finish();
            }
            HttpResponse::Ok().finish()
        }
        None => {
            // Invalid Token
            tracing::error!("Invalid token");
            HttpResponse::Unauthorized().finish()
        }
    }
}

#[tracing::instrument(
    name = "Get subscriber_id from token"
    skip(subscription_token, pool)
)]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &SubscriptionToken,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT subscriber_id FROM subscription_tokens
        WHERE subscription_token = $1
        "#,
        subscription_token.as_ref()
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(result.map(|r| r.subscriber_id))
}

#[tracing::instrument(
    name = "Mark subscriber as confirmed"
    skip(subscriber_id, pool)
)]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE subscriptions SET status = 'confirmed' WHERE id = $1
        "#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
