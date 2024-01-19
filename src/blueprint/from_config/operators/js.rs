use crate::blueprint::*;
use crate::config;
use crate::config::{Config, Field};
use crate::javascript::{JsPluginWrapper, JsPluginWrapperInterface};
use crate::lambda::Lambda;
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn update_js(
  js_wrapper: &JsPluginWrapper,
) -> TryFold<'_, (&Config, &Field, &config::Type, &str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(|(_, field, _, _), b_field| {
    let mut updated_b_field = b_field;

    if let Some(op) = &field.js {
      #[cfg(not(feature = "unsafe-js"))]
      return Valid::fail("JS feature is disabled".to_string());

      let executor = js_wrapper.create_executor(op.inline.clone(), op.with_context);

      updated_b_field =
        updated_b_field.resolver_or_default(Lambda::context().to_js(executor.clone()), |r| r.to_js(executor.clone()));
    }
    Valid::succeed(updated_b_field)
  })
}
