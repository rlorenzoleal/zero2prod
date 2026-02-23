use super::SubscribeError;
use super::helpers::*;

use crate::domain::{
    NewSubscriber, NewSubscriberError, SubscriberEmail, SubscriberName, SubscriptionStatus,
    SubscriptionToken,
};
use crate::email_client::EmailClient;
use crate::startup::ApplicationBaseUrl;

use actix_web::{HttpResponse, web};
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = NewSubscriberError;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.name)?;
        let email = SubscriberEmail::parse(form.email)?;
        Ok(NewSubscriber { email, name })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client, base_url),
    fields (
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, SubscribeError> {
    let new_subscriber: NewSubscriber = form.0.try_into()?;

    let mut transaction = pool.begin().await?;

    let subscriber_id = {
        match get_subscriber_id_and_status_by_email(&pool, &new_subscriber.email).await? {
            Some((_, SubscriptionStatus::Confirmed)) => {
                tracing::info!("User is already subscribed");
                return Ok(HttpResponse::Ok().finish());
            }
            Some((uuid, SubscriptionStatus::PendingConfirmation)) => uuid,
            None => insert_subscriber(&mut transaction, &new_subscriber).await?,
        }
    };

    let subscription_token = SubscriptionToken::generate_random();

    store_token(&mut transaction, subscriber_id, &subscription_token).await?;

    transaction.commit().await?;

    send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await?;

    Ok(HttpResponse::Ok().finish())
}
