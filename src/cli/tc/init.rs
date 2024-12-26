use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Result};

use super::helpers::{GRAPHQL_RC, TAILCALL_RC, TAILCALL_RC_SCHEMA};
use crate::cli::runtime::{confirm_and_write, create_directory, select_prompt};
use crate::core::config::{
    Config, Expr, Field, Link, LinkType, Resolver, RootSchema, RuntimeConfig, Source,
};
use crate::core::merge_right::MergeRight;
use crate::core::runtime::TargetRuntime;
use crate::core::{config, Type};

const SCHEMA_FILENAME: &str = "main.graphql";

pub(super) async fn init_command(runtime: TargetRuntime, folder_path: &str) -> Result<()> {
    create_directory(folder_path).await?;

    let detected_configuration_format = detect_configuration_format(folder_path).map(Ok);

    let selection = detected_configuration_format.unwrap_or_else(|| {
        select_prompt(
            "Please select the format in which you want to generate the config.",
            vec![Source::Json, Source::Yml],
        )
    })?;

    let tailcallrc = include_str!("../../../generated/.tailcallrc.graphql");
    let tailcallrc_json: &str = include_str!("../../../generated/.tailcallrc.schema.json");

    let tailcall_rc = Path::new(folder_path).join(TAILCALL_RC);
    let tailcall_rc_schema = Path::new(folder_path).join(TAILCALL_RC_SCHEMA);
    let graphql_rc = Path::new(folder_path).join(GRAPHQL_RC);

    match selection {
        Source::GraphQL => {
            // .tailcallrc.graphql
            runtime
                .file
                .write(&tailcall_rc.display().to_string(), tailcallrc.as_bytes())
                .await?;

            // .graphqlrc.yml
            confirm_and_write_yml(runtime.clone(), &graphql_rc).await?;
        }

        Source::Json | Source::Yml => {
            // .tailcallrc.schema.json
            runtime
                .file
                .write(
                    &tailcall_rc_schema.display().to_string(),
                    tailcallrc_json.as_bytes(),
                )
                .await?;
        }
    }

    create_main(runtime.clone(), folder_path, selection).await?;

    Ok(())
}

fn default_graphqlrc() -> serde_yaml_ng::Value {
    serde_yaml_ng::Value::Mapping(serde_yaml_ng::mapping::Mapping::from_iter([(
        "schema".into(),
        serde_yaml_ng::Value::Sequence(vec!["./.tailcallrc.graphql".into(), "./*.graphql".into()]),
    )]))
}

async fn confirm_and_write_yml(
    runtime: TargetRuntime,
    yml_file_path: impl AsRef<Path>,
) -> Result<()> {
    let yml_file_path = yml_file_path.as_ref().display().to_string();

    let mut final_graphqlrc = default_graphqlrc();

    match runtime.file.read(yml_file_path.as_ref()).await {
        Ok(yml_content) => {
            let graphqlrc: serde_yaml_ng::Value = serde_yaml_ng::from_str(&yml_content)?;
            final_graphqlrc = graphqlrc.merge_right(final_graphqlrc);
            let content = serde_yaml_ng::to_string(&final_graphqlrc)?;
            confirm_and_write(runtime.clone(), &yml_file_path, content.as_bytes()).await
        }
        Err(_) => {
            let content = serde_yaml_ng::to_string(&final_graphqlrc)?;
            runtime.file.write(&yml_file_path, content.as_bytes()).await
        }
    }
}

fn main_config() -> Config {
    let field = Field {
        type_of: Type::from("String".to_owned()).into_required(),
        resolvers: Resolver::Expr(Expr { body: "Hello, World!".into() }).into(),
        ..Default::default()
    };

    let query_type = config::Type {
        fields: BTreeMap::from([("greet".into(), field)]),
        ..Default::default()
    };

    Config {
        schema: RootSchema { query: Some("Query".to_string()), ..Default::default() },
        types: BTreeMap::from([("Query".into(), query_type)]),
        ..Default::default()
    }
}

fn runtime_config() -> RuntimeConfig {
    let config = RuntimeConfig::default();

    config.links(vec![Link {
        id: Some("main".to_string()),
        src: SCHEMA_FILENAME.to_string(),
        type_of: LinkType::Config,
        ..Default::default()
    }])
}

async fn create_main(
    runtime: TargetRuntime,
    folder_path: impl AsRef<Path>,
    source: Source,
) -> Result<()> {
    let path = folder_path
        .as_ref()
        .join(format!("main.{}", source.ext()))
        .display()
        .to_string();

    // check if the main file already exists and skip creation
    if std::fs::metadata(&path).is_ok() {
        return Ok(());
    }

    let config = main_config();
    let runtime_config = runtime_config();

    let runtime_config = match source {
        Source::Json => runtime_config.to_json(true)?,
        Source::Yml => runtime_config.to_yaml()?,
        _ => {
            return Err(anyhow!(
                "Only json/yaml formats are supported for json configs"
            ))
        }
    };

    let schema = config.to_sdl();

    let runtime_config_path = folder_path
        .as_ref()
        .join(format!("main.{}", source.ext()))
        .display()
        .to_string();
    let schema_path = folder_path
        .as_ref()
        .join(SCHEMA_FILENAME)
        .display()
        .to_string();

    confirm_and_write(
        runtime.clone(),
        &runtime_config_path,
        runtime_config.as_bytes(),
    )
    .await?;
    confirm_and_write(runtime.clone(), &schema_path, schema.as_bytes()).await?;

    Ok(())
}

/// Used to detect the configuration format of the tailcallrc file in the given
/// folder. This is useful in situations where tailcall configuration was
/// initialized already.
fn detect_configuration_format(folder_path: impl AsRef<Path>) -> Option<Source> {
    let folder_path = folder_path.as_ref();
    let json_path = folder_path.join(".tailcallrc.schema.json");
    let yaml_path = folder_path.join(".tailcallrc.schema.yaml");
    let yml_path = folder_path.join(".tailcallrc.schema.yml");
    let graphql_path = folder_path.join(".tailcallrc.schema.graphql");

    if json_path.exists() {
        return Some(Source::Json);
    } else if yaml_path.exists() || yml_path.exists() {
        return Some(Source::Yml);
    } else if graphql_path.exists() {
        return Some(Source::GraphQL);
    }

    None
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_detect_configuration_format() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        // Test JSON configuration detection
        let json_path = dir_path.join(".tailcallrc.schema.json");
        fs::write(&json_path, "").unwrap();
        assert_eq!(detect_configuration_format(dir_path), Some(Source::Json));
        fs::remove_file(&json_path).unwrap();

        // Test YAML configuration detection
        let yaml_path = dir_path.join(".tailcallrc.schema.yaml");
        fs::write(&yaml_path, "").unwrap();
        assert_eq!(detect_configuration_format(dir_path), Some(Source::Yml));
        fs::remove_file(&yaml_path).unwrap();

        // Test YML configuration detection
        let yml_path = dir_path.join(".tailcallrc.schema.yml");
        fs::write(&yml_path, "").unwrap();
        assert_eq!(detect_configuration_format(dir_path), Some(Source::Yml));
        fs::remove_file(&yml_path).unwrap();

        // Test GraphQL configuration detection
        let graphql_path = dir_path.join(".tailcallrc.schema.graphql");
        fs::write(&graphql_path, "").unwrap();
        assert_eq!(detect_configuration_format(dir_path), Some(Source::GraphQL));
        fs::remove_file(&graphql_path).unwrap();

        // Test no configuration detection
        assert_eq!(detect_configuration_format(dir_path), None);
    }
}
