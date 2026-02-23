use unicode_segmentation::UnicodeSegmentation;

const NAME_LENGTH_LIMIT: usize = 256;

#[derive(Debug)]
pub struct SubscriberName(String);

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum SubscriberNameError {
    #[error("Subscriber name can't be empty")]
    EmptyOrWhiteSpace,
    #[error("Subscriber name length ({0}) exceeds the {limit} graphemes limit", limit = NAME_LENGTH_LIMIT)]
    TooLong(usize),
    #[error("Subscriber name contains an invalid character: {0}")]
    InvalidCharacter(char),
}

impl SubscriberName {
    pub fn parse(s: String) -> Result<Self, SubscriberNameError> {
        if s.trim().is_empty() {
            return Err(SubscriberNameError::EmptyOrWhiteSpace);
        }

        let graphemes_count = s.graphemes(true).count();
        if graphemes_count > NAME_LENGTH_LIMIT {
            return Err(SubscriberNameError::TooLong(graphemes_count));
        }

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

        if let Some(forbidden_char) = s.chars().find(|c| forbidden_characters.contains(c)) {
            return Err(SubscriberNameError::InvalidCharacter(forbidden_char));
        }

        Ok(Self(s))
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use crate::domain::subscriber_name::SubscriberNameError;

    use super::{NAME_LENGTH_LIMIT, SubscriberName};
    use claims::{assert_err_eq, assert_ok};

    #[test]
    fn a_name_as_long_as_the_limit_is_valid() {
        let name = "ë".repeat(NAME_LENGTH_LIMIT);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_longer_than_the_limit_is_rejected() {
        let name = "a".repeat(NAME_LENGTH_LIMIT + 1);
        assert_err_eq!(
            SubscriberName::parse(name),
            SubscriberNameError::TooLong(NAME_LENGTH_LIMIT + 1)
        );
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err_eq!(
            SubscriberName::parse(name),
            SubscriberNameError::EmptyOrWhiteSpace
        );
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err_eq!(
            SubscriberName::parse(name),
            SubscriberNameError::EmptyOrWhiteSpace
        );
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for char in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = char.to_string();
            assert_err_eq!(
                SubscriberName::parse(name),
                SubscriberNameError::InvalidCharacter(*char)
            );
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
