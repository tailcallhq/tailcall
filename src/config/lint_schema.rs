use super::Source;
use crate::config::Config;
use colored::Colorize;
use regex::Regex;

pub struct LintSchema {}
impl LintSchema {
    // helper functions
    fn is_pascal_case(string_to_check: &str) -> bool {
        let case_regex = Regex::new("^[A-Z]([A-Za-z]*)$");
        match case_regex {
            Ok(re) => re.is_match(string_to_check),
            Err(_) => false,
        }
    }
    fn is_camel_case(string_to_check: &str) -> bool {
        let case_regex: Result<_, _> = Regex::new("^[a-z]([A-Za-z]*)$");
        match case_regex {
            Ok(re) => re.is_match(string_to_check),
            Err(_) => false,
        }
    }
    fn is_all_caps_case(string_to_check: &str) -> bool {
        let case_regex: Result<_, _> = Regex::new("^[A-Z]([A-Z]*)$");
        match case_regex {
            Ok(re) => re.is_match(string_to_check),
            Err(_) => false,
        }
    }

    fn show_lint_errors(config: Config, file_path: &str) {
        // warn for lint errors and stop at each file.

        // store warnings to print at the end of each file.
        let mut warnings: Vec<String> = vec![];

        // main logic
        for name in config.types.keys() {
            if let Some(temp1) = config.types.get(name) {
                if !Self::is_pascal_case(name) {
                    if let Some(_) = &temp1.variants {
                        warnings.push(
                            "Enum Name : ".to_string()
                                + name
                                + " --> Enum Name should be in PascalCase.",
                        );
                    } else {
                        warnings.push(
                            "Type Name : ".to_string()
                                + name
                                + " --> Type Name should be in PascalCase.",
                        );
                    }
                }

                for field_name in temp1.fields.keys() {
                    if !Self::is_camel_case(field_name) {
                        warnings.push(
                            "Type Name : ".to_string()
                                + name
                                + "; Field Name : "
                                + field_name
                                + " --> Field Name should be in camelCase.",
                        );
                    }
                }

                // for enums type
                if let Some(variants) = &temp1.variants {
                    for variant in variants {
                        if !Self::is_all_caps_case(variant) {
                            warnings.push(
                                "Enum Name : ".to_string()
                                    + name
                                    + "; Enum Value : "
                                    + variant
                                    + " --> Enum Value should be in ALL CAPS Case.",
                            );
                        }
                    }
                }
            }
        }

        let no_of_warnings = warnings.len();

        println!(
            "{} : {}",
            "\nLint Schema Results for schema in file Path".yellow(),
            file_path.blue()
        );

        if no_of_warnings > 0 {
            println!("{}", warnings.join("\n").red());
        }
        if no_of_warnings == 0 {
            println!("{}", "No Lint errors in schema.".green());
        }

        println!("{}", "End of Lint Schema Results.\n".yellow());
    }

    fn auto_fix(config: Config) -> Config {
        let modified_config = config.clone();
        println!(
            "{}\n{}",
            "AutoFix :", "We can modify config here if any lint errors."
        );
        // autofix all the linting errors and show the changes to the user.
        // it involves making changes to config and going for build with new config.

        // situation where we can change.
        // 1. camel, pascal => ALL CAPS
        // 2. camel => PascalCase
        // 3. PascalCase => camelCase

        // main logic
        for name in config.types.keys() {
            if let Some(temp1) = config.types.get(name) {
                if !Self::is_camel_case(name) {
                    if let Some(_) = &temp1.variants {
                        // enum
                        // change starting letter to CAPITAL
                    } else {
                        // type
                    }
                }

                for field_name in temp1.fields.keys() {
                    if !Self::is_camel_case(field_name) {
                        // type names, values
                    }
                }

                // for enums type
                if let Some(variants) = &temp1.variants {
                    for variant in variants {
                        if !Self::is_all_caps_case(variant) {
                            // enum names, values
                        }
                    }
                }
            }
        }

        modified_config
    }

    pub fn lint(config: Config, file_path: &str, source: Source) -> Config {
        let auto_fix_runnable = source.ext() == "graphql";

        Self::show_lint_errors(config.clone(), file_path);

        // config.lint: {field: true, type: true, enum: true, enumValue: true, autoFix: false}
        if config.server.lint.auto_fix && auto_fix_runnable {
            Self::auto_fix(config.clone())
        } else {
            config
        }
    }
}
