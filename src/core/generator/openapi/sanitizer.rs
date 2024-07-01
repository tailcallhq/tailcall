use crate::core::config::Config;
use crate::core::generator::openapi::helpers::TYPE_FIELD;
use crate::core::transform::Transform;
use crate::core::valid::Valid;

pub struct Sanitizer;

impl Transform for Sanitizer {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        for (_, typ) in config.types.iter_mut() {
            if let Some(field) = typ.fields.remove("type") {
                typ.fields.insert(TYPE_FIELD.to_string(), field);
            }
            typ.fields.iter_mut().for_each(|(_, fld)| {
                if let Some(arg) = fld.args.remove("type") {
                    fld.args.insert(TYPE_FIELD.to_string(), arg);
                }
                if let Some(http) = fld.http.as_mut() {
                    http.path = http
                        .path
                        .replace("{{.args.type}}", &format!("{{{{.args.{TYPE_FIELD}}}}}"));
                    http.query.iter_mut().for_each(|keyvalue| {
                        keyvalue.value = keyvalue
                            .value
                            .replace("{{.args.type}}", &format!("{{{{.args.{TYPE_FIELD}}}}}"));
                    })
                }
            })
        }

        Valid::succeed(config)
    }
}
