use async_graphql::validators::email;
use async_graphql_value::ConstValue;

use crate::json::JsonLike;

#[derive(schemars::JsonSchema)]
/// A custom scalar to validate the format of email address
pub struct Email;

impl Email {
    /// Function used to validate the email address
    pub fn validate(value: &ConstValue) -> bool {
        println!("h: {}", value);
        if let Ok(email_str) = value.clone().as_str_ok() {
            let email_str = email_str.to_string();
            return email(&email_str).is_ok();
        }
        false
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use async_graphql::dynamic;
    use async_graphql::dynamic::Type::Object;
    use async_graphql::dynamic::{Field, FieldFuture, InputValue, TypeRef};
    use async_graphql_value::value;

    use crate::scalars::Email;

    fn get_schema(scalar: dynamic::Scalar, resp: String) -> Result<dynamic::Schema> {
        // define scalar
        // equivalent to
        // scalar Email
        //
        // type Query {
        //  value(val: Email!): Email!
        // }

        let mut schema = dynamic::Schema::build("Query", None, None);
        let mut object = dynamic::Object::new("Query");
        let mut field = Field::new("value", TypeRef::named_nn("Email"), move |_| {
            let resp = resp.clone();
            FieldFuture::new(async move { Ok(Some(value!(resp))) })
        });

        field = field.argument(InputValue::new("val", TypeRef::named_nn("Email")));

        object = object.field(field);

        schema = schema.register(Object(object));

        schema = schema.register(dynamic::Type::Scalar(scalar));

        Ok(schema.finish()?)
    }

    #[tokio::test]
    async fn test_email_valid_req_resp() -> Result<()> {
        // define and add validator for email
        let mut scalar = dynamic::Scalar::new("Email");
        scalar = scalar.validator(Email::validate);

        let response_body = "alo@validresp.com".to_string();
        let schema = get_schema(scalar, response_body.clone())?;

        let resp = schema.execute("{ value(val: \"alo@valid.com\") }").await;

        assert_eq!(
            response_body.as_str(),
            resp.data
                .into_json()
                .unwrap()
                .as_object()
                .unwrap()
                .get("value")
                .unwrap()
                .as_str()
                .unwrap()
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_email_invalid() -> Result<()> {
        // define and add validator for email
        let mut scalar = dynamic::Scalar::new("Email");
        scalar = scalar.validator(Email::validate);

        let schema = get_schema(scalar, "alo@validresp.com".to_string())?;

        let resp = schema.execute("{ value(val: \"alo@invalidvalid\") }").await;

        assert!(resp.data.into_json().unwrap().is_null());

        Ok(())
    }
}
