use async_graphql::validators::email;
use async_graphql_value::ConstValue;

use crate::json::JsonLike;

#[derive(schemars::JsonSchema)]
/// field whose value conforms to the standard internet email address format as
/// specified in HTML Spec: https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address.
pub struct Email;

impl super::Scalar for Email {
    /// Function used to validate the email address
    fn validate(&self) -> fn(&ConstValue) -> bool {
        |value| {
            if let Ok(email_str) = value.clone().as_str_ok() {
                let email_str = email_str.to_string();
                return email(&email_str).is_ok();
            }
            false
        }
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use async_graphql_value::ConstValue;

    use crate::scalars::{Email, Scalar};

    #[tokio::test]
    async fn test_email_valid_req_resp() -> Result<()> {
        assert!(Email.validate()(&ConstValue::String(
            "valid@email.com".to_string()
        )));
        Ok(())
    }

    #[tokio::test]
    async fn test_email_invalid() -> Result<()> {
        assert!(!Email.validate()(&ConstValue::String(
            "invalid_email".to_string()
        )));
        Ok(())
    }

    #[tokio::test]
    async fn test_email_invalid_const_value() -> Result<()> {
        assert!(!Email.validate()(&ConstValue::Null));
        Ok(())
    }
}
