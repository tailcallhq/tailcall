use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, schemars::JsonSchema)]
pub enum TextCase {
    Camel,
    Pascal,
    Snake,
    ScreamingSnake,
}

/// The @lint directive allows you to configure linting.
#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug, Clone, schemars::JsonSchema)]
pub struct Lint {
    ///
    /// To autoFix the lint.
    /// Example Usage lint:{autoFix:true}
    #[serde(rename = "autoFix")]
    pub auto_fix: Option<bool>,
    ///
    ///
    /// This enum is provided with appropriate TextCase.
    /// Example Usage: lint:{enum:Pascal}
    #[serde(rename = "enum")]
    pub enum_lint: Option<TextCase>,
    ///
    ///
    /// This enumValue is provided with appropriate TextCase.
    /// Example Usage: lint:{enumValue:ScreamingSnake}
    #[serde(rename = "enumValue")]
    pub enum_value_lint: Option<TextCase>,
    ///
    ///
    /// This field is provided with appropriate TextCase.
    /// Example Usage: lint:{field:Camel}
    #[serde(rename = "field")]
    pub field_lint: Option<TextCase>,
    ///
    ///
    /// This type is provided with appropriate TextCase.
    /// Example Usage: lint:{type:Pascal}
    #[serde(rename = "type")]
    pub type_lint: Option<TextCase>,
}

impl Lint {
    // helper function
    fn handle_case(value: TextCase) -> Case {
        match value {
            TextCase::Camel => Case::Camel,
            TextCase::Pascal => Case::Pascal,
            TextCase::Snake => Case::Snake,
            TextCase::ScreamingSnake => Case::ScreamingSnake,
        }
    }

    fn show_and_or_auto_fix(mut config: Config, lint: Lint, auto_fix: bool) -> Config {
        let mut may_be_modified_config = config.clone();

        let mut has_lint_problems = false;
        // autofix all the linting errors and show the changes to the user.
        // it involves making changes to config and going for build with new config.

        // type_of
        for (type_name, type_data) in config.types.iter_mut() {
            for (field_name, field_data) in type_data.clone().fields.iter() {
                let mut may_be_modified_field_data = field_data.clone();

                if may_be_modified_config
                    .types
                    .contains_key(&field_data.type_of)
                {
                    if let Some(value) = lint.type_lint.clone() {
                        let case = Self::handle_case(value);
                        if !field_data.type_of.is_case(case) {
                            tracing::warn!("{}", format!("Type Name: {}, Field Name : {}, type_of : {} | type_of is some Type Name, which should be in {:?} case", type_name, field_name, &field_data.type_of, case));

                            if auto_fix {
                                let modified_type_of = field_data.type_of.to_case(case);
                                may_be_modified_field_data = may_be_modified_field_data
                                    .clone()
                                    .type_of(modified_type_of.clone());
                                type_data
                                    .fields
                                    .insert(field_name.to_string(), may_be_modified_field_data);
                                tracing::info!(
                                    "{}",
                                    format!(
                                        "Auto-fixed the type_of '{}' to '{}'",
                                        &field_data.type_of, &modified_type_of
                                    )
                                );
                            }
                        }
                    }
                }
            }

            may_be_modified_config
                .types
                .insert(type_name.to_string(), type_data.clone());
        }

        // main logic
        for (type_name, type_data) in config.types.iter_mut() {
            let mut may_be_modified_type_name = type_name.clone();

            // type names
            if let Some(value) = lint.type_lint.clone() {
                if type_data.variants.is_some() {
                } else {
                    let case = Self::handle_case(value);
                    if !type_name.is_case(case) {
                        has_lint_problems = true;
                        // warn user
                        tracing::warn!(
                            "{}",
                            format!(
                                "Type Name : {} | Type Name should be in {:?} case",
                                type_name, case
                            )
                        );

                        if auto_fix {
                            may_be_modified_config.types.remove(type_name);
                            may_be_modified_type_name = type_name.to_case(case).to_string();
                            tracing::info!(
                                "{}",
                                format!(
                                    "Auto-fixed the type name '{}' to '{}'",
                                    type_name, &may_be_modified_type_name
                                )
                            );
                        }
                    }
                }
            }

            // field names
            for (field_name, field_data) in type_data.fields.clone().iter() {
                if let Some(value) = lint.field_lint.clone() {
                    let case = Self::handle_case(value);

                    if !field_name.is_case(case) {
                        has_lint_problems = true;
                        // warn the user
                        tracing::warn!("{}", format!("Type Name : {}, Field Name : {} | Field Name should be in {:?} case", type_name, field_name, case));

                        if auto_fix {
                            let modified_field_name = field_name.to_case(case);

                            type_data
                                .fields
                                .insert(modified_field_name.clone(), field_data.clone());
                            type_data.fields.remove(field_name);
                            tracing::info!(
                                "{}",
                                format!(
                                    "Auto-fixed the field name '{}' to '{}'",
                                    field_name, &modified_field_name
                                )
                            );
                        }
                    }
                }
            }

            // enum names
            if let Some(variants) = &mut type_data.variants {
                // handle enum type name
                if let Some(value) = lint.enum_lint.clone() {
                    let case = Self::handle_case(value);
                    if !type_name.is_case(case) {
                        has_lint_problems = true;
                        // warn user
                        tracing::warn!(
                            "{}",
                            format!(
                                "Enum Name : {} | Enum Name should be in {:?} case",
                                type_name, case
                            )
                        );

                        if auto_fix {
                            may_be_modified_config.types.remove(type_name);
                            may_be_modified_type_name = type_name.to_case(case).to_string();
                            tracing::info!(
                                "{}",
                                format!(
                                    "Auto-fixed the enum name '{}' to '{}'",
                                    type_name, &may_be_modified_type_name
                                )
                            );
                        }
                    }
                }

                // handle enum variants name
                let mut c_variants = variants.clone();
                for variant in variants.iter() {
                    if let Some(value) = lint.enum_value_lint.clone() {
                        let case = Self::handle_case(value);
                        if !variant.is_case(case) {
                            has_lint_problems = true;
                            // warn user
                            tracing::warn!("{}", format!("Enum Name : {}, Enum Value : {} | Enum Value should be in {:?} case", type_name, variant, case));

                            if auto_fix {
                                let modified_variant = variant.to_case(case);
                                c_variants.insert(modified_variant.clone());
                                c_variants.remove(variant);
                                tracing::info!(
                                    "{}",
                                    format!(
                                        "Auto-fixed the enum value '{}' to '{}'",
                                        variant, &modified_variant
                                    )
                                );
                            }
                        }
                    }
                }
                type_data.variants = Some(c_variants);
            }
            may_be_modified_config
                .types
                .insert(may_be_modified_type_name, type_data.clone());
        }

        if has_lint_problems && !auto_fix {
            tracing::info!("User may correct the lint problems with @modify operator.")
        }
        may_be_modified_config
    }

    pub fn lint(config: Config, lint: Lint) -> Config {
        // Check if user configured to autofix lint problems otherwise
        // just warn lint problems to the user.
        // example lint config ->
        // lint: {field: Camel, type: Pascal, enum: Pascal, enumValue: ScreamingSnake,
        // autoFix: false}
        let mut auto_fix = false;

        if let Some(value) = lint.auto_fix {
            auto_fix = value;
        }
        Self::show_and_or_auto_fix(config.clone(), lint, auto_fix)
    }
}

#[cfg(test)]
mod lint_tests {
    use crate::config::reader::ConfigReader;

    #[tokio::test]
    async fn test_lint() {
        let runtime = crate::runtime::test::init(None);
        let file = "examples/lint-auto-fix.graphql";
        let cr = ConfigReader::init(runtime);
        let c = cr.read(&file).await.unwrap();

        let mut type_linted = false;
        let mut field_linted = false;
        if let Some(user_type_data) = c.types.get("User") {
            type_linted = true;
            let user_type_fields = user_type_data.clone().fields;
            if user_type_fields.contains_key("username") {
                field_linted = true;
            }
        }

        let mut enum_type_linted = false;
        let mut variant_linted = false;
        if let Some(user_type_data) = c.types.get("PostType") {
            enum_type_linted = true;
            if let Some(user_type_variants) = user_type_data.clone().variants {
                if user_type_variants.contains("BLOG_POST") {
                    variant_linted = true;
                }
            }
        }

        assert!(type_linted && field_linted && enum_type_linted && variant_linted);
    }
}
