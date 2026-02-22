use rand::distributions::Alphanumeric;
use rand::{Rng, thread_rng};

const TOKEN_LENGTH: usize = 25;

#[derive(Debug)]
pub struct SubscriptionToken(String);

impl SubscriptionToken {
    pub fn parse(s: String) -> Result<Self, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_not_right_length = s.chars().count() != TOKEN_LENGTH;
        let is_not_alphanumeric = !s.chars().all(char::is_alphanumeric);

        if is_empty_or_whitespace || is_not_right_length || is_not_alphanumeric {
            Err(format!("{} is not a valid token", s))
        } else {
            Ok(Self(s))
        }
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
    use super::{SubscriptionToken, TOKEN_LENGTH};
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_token_of_the_right_length_is_valid() {
        let token = SubscriptionToken::generate_random_alphanumeric_string(TOKEN_LENGTH);
        assert_ok!(SubscriptionToken::parse(token));
    }

    #[test]
    fn a_token_smaller_than_the_right_length_is_rejected() {
        let token = SubscriptionToken::generate_random_alphanumeric_string(TOKEN_LENGTH - 1);
        assert_err!(SubscriptionToken::parse(token));
    }

    #[test]
    fn a_token_bigger_than_the_right_length_is_rejected() {
        let token = SubscriptionToken::generate_random_alphanumeric_string(TOKEN_LENGTH + 1);
        assert_err!(SubscriptionToken::parse(token));
    }

    #[test]
    fn whitespace_only_tokens_are_rejected() {
        let token = " ".repeat(TOKEN_LENGTH);
        assert_err!(SubscriptionToken::parse(token));
    }

    #[test]
    fn empty_tokens_are_rejected() {
        let token = "".to_string();
        assert_err!(SubscriptionToken::parse(token));
    }

    #[test]
    fn non_alphanumeric_tokens_are_rejected() {
        let mut token = SubscriptionToken::generate_random_alphanumeric_string(TOKEN_LENGTH - 1);
        token.push('*');
        assert_err!(SubscriptionToken::parse(token));
    }
}
