use crate::domain::{SubscriberEmail, SubscriberEmailError, SubscriberName, SubscriberNameError};

pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

#[derive(thiserror::Error, Debug)]
pub enum NewSubscriberError {
    #[error("Invalid subscriber email: {0:?}")]
    InvalidEmail(#[from] SubscriberEmailError),
    #[error("Invalid subscriber name: {0:?}")]
    InvalidName(#[from] SubscriberNameError),
}
