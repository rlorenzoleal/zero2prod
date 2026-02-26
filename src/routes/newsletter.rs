use crate::domain::SubscriberEmail;
use crate::domain::SubscriberEmailError;
use crate::email_client::EmailClient;
use crate::telemetry::spawn_blocking_with_tracing;

use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::ResponseError;
use actix_web::http::StatusCode;
use actix_web::http::header::{self, HeaderMap, HeaderValue};
use actix_web::web;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use base64::Engine;
use secrecy::ExposeSecret;
use secrecy::Secret;
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

#[allow(dead_code)]
struct Credentials {
    username: String,
    password: Secret<String>,
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
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    #[error("Password error: {0}")]
    PasswordError(#[from] argon2::password_hash::Error),
    #[error("Failed to spawn blocking task: {0}")]
    SpawnError(#[from] tokio::task::JoinError),
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse {
        match self {
            PublishError::DatabaseError(_)
            | PublishError::NewsletterPublicationError(_)
            | PublishError::SpawnError(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
            PublishError::AuthenticationError(_) | PublishError::PasswordError(_) => {
                let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();

                response
                    .headers_mut()
                    .insert(header::WWW_AUTHENTICATE, header_value);

                response
            }
        }
    }
}

#[tracing::instrument(
    name = "Publish newsletter"
    skip(body, pool, email_client, request)
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    request: HttpRequest,
) -> Result<HttpResponse, PublishError> {
    let credentials = basic_authentication(request.headers())?;

    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    let user_id = validate_credentials(credentials, &pool).await?;

    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

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

fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, PublishError> {
    let header_value = headers
        .get("Authorization")
        .ok_or_else(|| auth_error("The 'Authorization' header was missing."))?
        .to_str()
        .map_err(|_err| auth_error("The 'Authorization' header was not a valid UTF8 string."))?;

    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .ok_or_else(|| auth_error("The authorization scheme was not 'Basic'."))?;

    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_segment)
        .map_err(|err| {
            auth_error(&format!(
                "Failed to base64-decode 'Basic' credentials: {err}"
            ))
        })?;

    let decoded_credentials = String::from_utf8(decoded_bytes)
        .map_err(|_err| auth_error("The decoded credential string is not valid UTF8."))?;

    let mut credentials = decoded_credentials.splitn(2, ':');

    let username = credentials
        .next()
        .ok_or_else(|| auth_error("A username must be provided in 'Basic' auth."))?
        .to_string();

    if username.is_empty() {
        return Err(auth_error("username can't be empty"));
    }

    let password = credentials
        .next()
        .ok_or_else(|| auth_error("A password must be provided in 'Basic' auth."))?
        .to_string();

    if password.is_empty() {
        return Err(auth_error("password can't be empty"));
    }

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}

fn auth_error(msg: &str) -> PublishError {
    PublishError::AuthenticationError(msg.to_string())
}

#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool,
) -> Result<uuid::Uuid, PublishError> {
    let mut user_id = None;
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .to_string(),
    );

    if let Some((stored_user_id, stored_password_hash)) =
        get_stored_credentials(&credentials.username, pool).await?
    {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_password_hash;
    }

    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_password_hash, credentials.password)
    })
    .await??;

    // Return user_id if found (Some), error if not
    user_id.ok_or_else(|| auth_error("Unknown username"))
}

#[tracing::instrument(
    name = "Verify password hash",
    skip(expected_password_hash, password_candidate)
)]
fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<(), PublishError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .map_err(PublishError::PasswordError)
}

#[tracing::instrument(name = "Get stored credentials", skip(username, pool))]
async fn get_stored_credentials(
    username: &str,
    pool: &PgPool,
) -> Result<Option<(uuid::Uuid, Secret<String>)>, PublishError> {
    let row: Option<_> = sqlx::query!(
        r#"
        SELECT user_id, password_hash
        FROM users
        WHERE username = $1
        "#,
        username
    )
    .fetch_optional(pool)
    .await
    .map_err(|err| {
        tracing::error!("Failed to get stored credentials: {:?}", err);
        err
    })?
    .map(|row| (row.user_id, Secret::new(row.password_hash)));

    Ok(row)
}
