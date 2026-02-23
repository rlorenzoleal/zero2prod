use validator::ValidateEmail;

#[derive(Debug)]
pub struct SubscriberEmail(String);

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum SubscriberEmailError {
    #[error("{0} is not a valid subscriber email")]
    InvalidEmail(String),
}

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<Self, SubscriberEmailError> {
        if s.validate_email() {
            Ok(Self(s))
        } else {
            Err(SubscriberEmailError::InvalidEmail(s))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use crate::domain::subscriber_email::SubscriberEmailError;

    use super::SubscriberEmail;
    use claims::assert_err_eq;
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let email = SafeEmail().fake_with_rng(g);
            Self(email)
        }
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err_eq!(
            SubscriberEmail::parse(email.clone()),
            SubscriberEmailError::InvalidEmail(email)
        );
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert_err_eq!(
            SubscriberEmail::parse(email.clone()),
            SubscriberEmailError::InvalidEmail(email)
        );
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err_eq!(
            SubscriberEmail::parse(email.clone()),
            SubscriberEmailError::InvalidEmail(email)
        );
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid_email.0).is_ok()
    }
}
