use rand::distributions::Alphanumeric;
use rand::{Rng, thread_rng};

const TOKEN_LENGTH: usize = 25;

#[derive(Debug)]
pub struct SubscriptionToken(String);

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum SubscriptionTokenError {
    #[error("Subscription token can't be empty")]
    EmptyOrWhiteSpace,
    #[error("Subscription token length({0}) does not match the expected value: {expected}", expected = TOKEN_LENGTH)]
    InvalidLength(usize),
    #[error("Subscription token contain non-alphanumeric characters: {0}")]
    InvalidCharacter(String),
}

impl SubscriptionToken {
    pub fn parse(s: String) -> Result<Self, SubscriptionTokenError> {
        if s.trim().is_empty() {
            return Err(SubscriptionTokenError::EmptyOrWhiteSpace);
        }

        let chars_count = s.chars().count();
        if s.chars().count() != TOKEN_LENGTH {
            return Err(SubscriptionTokenError::InvalidLength(chars_count));
        }

        if !s.chars().all(char::is_alphanumeric) {
            return Err(SubscriptionTokenError::InvalidCharacter(s));
        }

        Ok(Self(s))
    }

    pub fn generate_random() -> Self {
        Self::parse(Self::generate_random_alphanumeric_string(TOKEN_LENGTH)).unwrap()
    }

    fn generate_random_alphanumeric_string(length: usize) -> String {
        let mut rng = thread_rng();
        std::iter::repeat_with(|| rng.sample(Alphanumeric))
            .map(char::from)
            .take(length)
            .collect()
    }
}

impl AsRef<str> for SubscriptionToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use super::{SubscriptionToken, SubscriptionTokenError, TOKEN_LENGTH};
    use claims::{assert_err_eq, assert_ok};

    #[test]
    fn a_token_of_the_right_length_is_valid() {
        let token = SubscriptionToken::generate_random_alphanumeric_string(TOKEN_LENGTH);
        assert_ok!(SubscriptionToken::parse(token));
    }

    #[test]
    fn a_token_smaller_than_the_right_length_is_rejected() {
        let token = SubscriptionToken::generate_random_alphanumeric_string(TOKEN_LENGTH - 1);
        assert_err_eq!(
            SubscriptionToken::parse(token),
            SubscriptionTokenError::InvalidLength(TOKEN_LENGTH - 1)
        );
    }

    #[test]
    fn a_token_bigger_than_the_right_length_is_rejected() {
        let token = SubscriptionToken::generate_random_alphanumeric_string(TOKEN_LENGTH + 1);
        assert_err_eq!(
            SubscriptionToken::parse(token),
            SubscriptionTokenError::InvalidLength(TOKEN_LENGTH + 1)
        );
    }

    #[test]
    fn whitespace_only_tokens_are_rejected() {
        let token = " ".repeat(TOKEN_LENGTH);
        assert_err_eq!(
            SubscriptionToken::parse(token),
            SubscriptionTokenError::EmptyOrWhiteSpace
        );
    }

    #[test]
    fn empty_tokens_are_rejected() {
        let token = "".to_string();
        assert_err_eq!(
            SubscriptionToken::parse(token),
            SubscriptionTokenError::EmptyOrWhiteSpace
        );
    }

    #[test]
    fn non_alphanumeric_tokens_are_rejected() {
        let mut token = SubscriptionToken::generate_random_alphanumeric_string(TOKEN_LENGTH - 1);
        token.push('*');
        assert_err_eq!(
            SubscriptionToken::parse(token.clone()),
            SubscriptionTokenError::InvalidCharacter(token)
        );
    }
}
