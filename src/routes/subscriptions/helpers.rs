use crate::domain::{NewSubscriber, SubscriberEmail, SubscriptionStatus, SubscriptionToken};
use crate::email_client::EmailClient;

use chrono::Utc;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[tracing::instrument(
    name = "Fetching subscriber details from database",
    skip(subscriber_email, pool)
)]
pub async fn get_subscriber_id_and_status_by_email(
    pool: &PgPool,
    subscriber_email: &SubscriberEmail,
) -> Result<Option<(Uuid, SubscriptionStatus)>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT id, status as "status: SubscriptionStatus"
        FROM subscriptions
        WHERE email = $1
        "#,
        subscriber_email.as_ref()
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get subscriber id and status: {:?}", e);
        e
    })?;

    Ok(result.map(|r| (r.id, r.status)))
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, transaction)
)]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    );

    transaction.execute(query).await.map_err(|err| {
        tracing::error!("Failed to insert subscriber: {:?}", err);
        err
    })?;

    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &SubscriptionToken,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"
        INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES($1, $2)
        "#,
        subscription_token.as_ref(),
        subscriber_id
    );

    transaction.execute(query).await.map_err(|err| {
        tracing::error!("Failed to store token: {:?}", err);
        err
    })?;

    Ok(())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, base_url, subscription_token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &SubscriptionToken,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url,
        subscription_token.as_ref()
    );
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
                Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(new_subscriber.email, "Welcome!", &html_body, &plain_body)
        .await
        .map_err(|err| {
            tracing::error!("Failed to send confirmation email {:?}", err);
            err
        })
}
