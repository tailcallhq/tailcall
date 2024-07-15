use std::collections::HashMap;

use crate::core::config::Config;
use crate::core::transform::Transform;
use crate::core::valid::{Valid, Validator};

/// Resolves the ambiguous types by renaming the input and
/// output types. The resolver function is called whenever is a conflict is
/// detected with the name that has a conflict. The returned value should return
/// a Resolution object containing the new input and output types.
/// The function will return a new ConfigModule with the resolved types.
pub struct Resolution {
    pub input: String,
    pub output: String,
}

impl Resolution {
    pub fn is_unique(&self) -> bool {
        self.input.ne(&self.output)
    }
}

pub struct AmbiguousType {
    resolver: Box<dyn Fn(&str) -> Resolution>,
}

impl Default for AmbiguousType {
    fn default() -> Self {
        Self::new(|v: &str| Resolution { input: format!("{}Input", v), output: v.to_owned() })
    }
}

impl AmbiguousType {
    pub fn new(resolver: impl Fn(&str) -> Resolution + 'static) -> Self {
        Self { resolver: Box::new(resolver) }
    }
}

fn insert_resolution(
    mut map: HashMap<String, Resolution>,
    current_name: &str,
    resolution: Resolution,
) -> HashMap<String, Resolution> {
    if resolution.input.eq(&resolution.output) {
        tracing::warn!(
            "Unable to resolve input and output type: {}",
            resolution.input
        );
    }

    if !map.contains_key(current_name) {
        map.entry(current_name.to_string()).or_insert(resolution);
    }

    map
}

impl Transform for AmbiguousType {
    type Value = Config;
    type Error = String;
    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        let mut input_types = config.input_types();
        let mut output_types = config.output_types();
        Valid::from_iter(input_types.intersection(&output_types), |current_name| {
            // Iterate over intersection of input and output types
            let resolution = (self.resolver)(current_name);

            if !resolution.is_unique() {
                Valid::fail(format!(
                    "Unable to auto resolve Input: {} and Output: {} are same",
                    resolution.input, resolution.output,
                ))
                .trace(current_name)
            } else {
                let mut resolution_map = HashMap::new();
                resolution_map = insert_resolution(resolution_map, current_name, resolution);
                if let Some(ty) = config.types.get(current_name) {
                    for field in ty.fields.values() {
                        for args in field.args.values() {
                            // if arg is of output type then it should be changed to that of
                            // newly created input type.
                            if output_types.contains(&args.type_of)
                                && !resolution_map.contains_key(&args.type_of)
                            {
                                let resolution = (self.resolver)(args.type_of.as_str());
                                resolution_map = insert_resolution(
                                    resolution_map,
                                    args.type_of.as_str(),
                                    resolution,
                                );
                            }
                        }
                    }
                }
                Valid::succeed(resolution_map)
            }
        })
        .map(|v| v.into_iter().flatten().collect::<HashMap<_, _>>())
        .map(|resolution_map| {
            // insert newly created types to the config.
            for (current_name, resolution) in &resolution_map {
                let input_name = &resolution.input;
                let output_name = &resolution.output;

                let og_ty = config.types.get(current_name).cloned();

                // remove old types
                config.types.remove(current_name);
                input_types.remove(current_name);
                output_types.remove(current_name);

                // add new types
                if let Some(og_ty) = og_ty {
                    config.types.insert(input_name.clone(), og_ty.clone());
                    input_types.insert(input_name.clone());

                    config.types.insert(output_name.clone(), og_ty);
                    output_types.insert(output_name.clone());
                }
            }
            let keys = config.types.keys().cloned().collect::<Vec<_>>();

            for k in keys {
                if let Some(ty) = config.types.get_mut(&k) {
                    for field in ty.fields.values_mut() {
                        if let Some(resolution) = resolution_map.get(&field.type_of) {
                            if output_types.contains(&k) {
                                field.type_of.clone_from(&resolution.output);
                            } else if input_types.contains(&k) {
                                field.type_of.clone_from(&resolution.input);
                            }
                        }
                        for arg in field.args.values_mut() {
                            if let Some(resolution) = resolution_map.get(&arg.type_of) {
                                arg.type_of.clone_from(&resolution.input);
                            }
                        }
                    }
                }
            }
            config
        })
    }
}

#[cfg(test)]
mod tests {

    use insta::assert_snapshot;
    use prost_reflect::prost_types::FileDescriptorSet;
    use tailcall_fixtures::protobuf;

    use crate::core::config::transformer::AmbiguousType;
    use crate::core::config::{Config, Type};
    use crate::core::generator::{Generator, Input};
    use crate::core::proto_reader::ProtoMetadata;
    use crate::core::transform::Transform;
    use crate::core::valid::Validator;

    fn build_qry(mut config: Config) -> Config {
        let mut query = Type::default();
        let mut field1 =
            crate::core::config::Field { type_of: "Type1".to_string(), ..Default::default() };

        let arg1 = crate::core::config::Arg { type_of: "Type1".to_string(), ..Default::default() };

        field1.args.insert("arg1".to_string(), arg1);

        let arg2 = crate::core::config::Arg { type_of: "Type2".to_string(), ..Default::default() };

        field1.args.insert("arg2".to_string(), arg2);

        let mut field2 = field1.clone();
        field2.type_of = "Type2".to_string();

        query.fields.insert("field1".to_string(), field1);
        query.fields.insert("field2".to_string(), field2);

        config.types.insert("Query".to_string(), query);
        config = config.query("Query");

        config
    }

    #[test]
    fn test_resolve_ambiguous_types() {
        // Create a ConfigModule instance with ambiguous types
        let mut config = Config::default();

        let mut type1 = Type::default();
        let mut type2 = Type::default();
        let mut type3 = Type::default();

        type1.fields.insert(
            "name".to_string(),
            crate::core::config::Field::default().type_of("String".to_string()),
        );

        type2.fields.insert(
            "ty1".to_string(),
            crate::core::config::Field::default().type_of("Type1".to_string()),
        );

        type3.fields.insert(
            "ty1".to_string(),
            crate::core::config::Field::default().type_of("Type1".to_string()),
        );
        type3.fields.insert(
            "ty2".to_string(),
            crate::core::config::Field::default().type_of("Type2".to_string()),
        );

        config.types.insert("Type1".to_string(), type1);
        config.types.insert("Type2".to_string(), type2);
        config.types.insert("Type3".to_string(), type3);

        config = build_qry(config);

        let config = AmbiguousType::default()
            .transform(config)
            .to_result()
            .unwrap();

        assert_snapshot!(config.to_sdl());
    }

    fn compile_protobuf(files: &[&str]) -> anyhow::Result<FileDescriptorSet> {
        Ok(protox::compile(files, [protobuf::SELF])?)
    }

    #[tokio::test]
    async fn test_resolve_ambiguous_news_types() -> anyhow::Result<()> {
        let news_proto = tailcall_fixtures::protobuf::NEWS;
        let set = compile_protobuf(&[protobuf::NEWS])?;

        let cfg_module = Generator::default()
            .inputs(vec![Input::Proto(ProtoMetadata {
                descriptor_set: set,
                path: news_proto.to_string(),
            })])
            .generate(false)?;

        let mut config = AmbiguousType::default()
            .transform(cfg_module.config().clone())
            .to_result()?;

        // remove links since they break snapshot tests
        config.links = Default::default();

        assert_snapshot!(config.to_sdl());
        Ok(())
    }
}
