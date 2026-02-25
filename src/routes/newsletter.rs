use crate::domain::SubscriberEmail;
use crate::domain::SubscriberEmailError;
use crate::email_client::EmailClient;

use actix_web::HttpResponse;
use actix_web::ResponseError;
use actix_web::http::StatusCode;
use actix_web::web;
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[derive(thiserror::Error, Debug)]
pub enum PublishError {
    #[error("Database error: {0:?}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Failed to send newsletter: {0:?}")]
    NewsletterPublicationError(#[from] reqwest::Error),
}

impl ResponseError for PublishError {
    fn status_code(&self) -> StatusCode {
        match self {
            PublishError::DatabaseError(_) | PublishError::NewsletterPublicationError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

#[tracing::instrument(
    name = "Publishing newsletter"
    skip(body, pool, email_client)
)]
pub async fn publish_newsletter(
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, PublishError> {
    let subscribers = get_confirmed_subscribers(&pool).await?;

    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                        &subscriber.email,
                        &body.title,
                        &body.content.html,
                        &body.content.text,
                    )
                    .await?;
            }
            Err(err) => {
                tracing::warn!(
                    "A confirmed subscriber is using an invalid email address: {}",
                    err
                );
            }
        }
    }
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, SubscriberEmailError>>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get confirmed subscribers {:?}", e);
        e
    })?;

    let confirmed_subscribers = rows
        .into_iter()
        .map(|r| SubscriberEmail::parse(r.email).map(|email| ConfirmedSubscriber { email }))
        .collect();

    Ok(confirmed_subscribers)
}
