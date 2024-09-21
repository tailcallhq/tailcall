use std::collections::HashMap;

use url::Url;

use crate::core::generator::Input;
use crate::core::valid::Valid;
use crate::core::Transform;

type SuggestedFieldName = String;

/// A transformer that adds suggested field names to the input_samples
pub struct RenameFields(HashMap<Url, Vec<SuggestedFieldName>>);

impl RenameFields {
    pub fn new(suggested_names: HashMap<Url, Vec<String>>) -> Self {
        Self(suggested_names)
    }
}

impl Transform for RenameFields {
    type Value = Vec<Input>;
    type Error = String;

    fn transform(&self, mut input_samples: Self::Value) -> Valid<Self::Value, Self::Error> {
        // ensure to choose only the suggested that has not been already used
        for (url_to_replace, suggested_names) in self.0.iter() {
            if let Some(name) = suggested_names.iter().find(|name| {
                !input_samples.iter().any(|input| {
                    if let Input::Json { field_name, .. } = input {
                        field_name.as_ref() == Some(name)
                    } else {
                        false
                    }
                })
            }) {
                if let Some(Input::Json {  field_name, .. }) = input_samples.iter_mut().find(|input| {
                    matches!(input, Input::Json { url: input_url, .. } if input_url == url_to_replace)
                }) {
                    field_name.replace(name.clone());
                }
            }
        }

        Valid::succeed(input_samples)
    }
}
