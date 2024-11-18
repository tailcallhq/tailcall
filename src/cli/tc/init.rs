use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;

use super::helpers::{GRAPHQL_RC, TAILCALL_RC, TAILCALL_RC_SCHEMA};
use crate::cli::runtime::{confirm_and_write, create_directory, select_prompt};
use crate::core::config::{Config, Expr, Field, Resolver, RootSchema, SourceUtil};
use crate::core::merge_right::MergeRight;
use crate::core::runtime::TargetRuntime;
use crate::core::{config, Type};

pub(super) async fn init_command(runtime: TargetRuntime, folder_path: &str) -> Result<()> {
    create_directory(folder_path).await?;

    let selection = select_prompt(
        "Please select the format in which you want to generate the config.",
        vec![SourceUtil::GraphQL, SourceUtil::Json, SourceUtil::Yml],
    )?;

    let tailcallrc = include_str!("../../../generated/.tailcallrc.graphql");
    let tailcallrc_json: &str = include_str!("../../../generated/.tailcallrc.schema.json");

    let tailcall_rc = Path::new(folder_path).join(TAILCALL_RC);
    let tailcall_rc_schema = Path::new(folder_path).join(TAILCALL_RC_SCHEMA);
    let graphql_rc = Path::new(folder_path).join(GRAPHQL_RC);

    match selection {
        SourceUtil::GraphQL => {
            // .tailcallrc.graphql
            confirm_and_write(
                runtime.clone(),
                &tailcall_rc.display().to_string(),
                tailcallrc.as_bytes(),
            )
            .await?;

            // .graphqlrc.yml
            confirm_and_write_yml(runtime.clone(), &graphql_rc).await?;
        }

        SourceUtil::Json | SourceUtil::Yml => {
            // .tailcallrc.schema.json
            confirm_and_write(
                runtime.clone(),
                &tailcall_rc_schema.display().to_string(),
                tailcallrc_json.as_bytes(),
            )
            .await?;
        }
    }

    create_main(runtime.clone(), folder_path, selection).await?;

    Ok(())
}

fn default_graphqlrc() -> serde_yaml::Value {
    serde_yaml::Value::Mapping(serde_yaml::mapping::Mapping::from_iter([(
        "schema".into(),
        serde_yaml::Value::Sequence(vec![
            "./.tailcallrc.graphql".into(),
            "./main.graphql".into(),
        ]),
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
            let graphqlrc: serde_yaml::Value = serde_yaml::from_str(&yml_content)?;
            final_graphqlrc = graphqlrc.merge_right(final_graphqlrc);
            let content = serde_yaml::to_string(&final_graphqlrc)?;
            confirm_and_write(runtime.clone(), &yml_file_path, content.as_bytes()).await
        }
        Err(_) => {
            let content = serde_yaml::to_string(&final_graphqlrc)?;
            runtime.file.write(&yml_file_path, content.as_bytes()).await
        }
    }
}

fn main_config() -> Config {
    let field = Field {
        type_of: Type::from("String".to_owned()).into_required(),
        resolver: Some(Resolver::Expr(Expr { body: "Hello, World!".into() })),
        ..Default::default()
    };

    let query_type = config::Type {
        fields: BTreeMap::from([("greet".into(), field)]),
        ..Default::default()
    };

    Config {
        server: Default::default(),
        upstream: Default::default(),
        schema: RootSchema { query: Some("Query".to_string()), ..Default::default() },
        types: BTreeMap::from([("Query".into(), query_type)]),
        ..Default::default()
    }
}

async fn create_main(
    runtime: TargetRuntime,
    folder_path: impl AsRef<Path>,
    source: SourceUtil,
) -> Result<()> {
    let config = main_config();

    let content = match source {
        SourceUtil::GraphQL => config.to_sdl(),
        SourceUtil::Json => config.to_json(true)?,
        SourceUtil::Yml => config.to_yaml()?,
    };

    let path = folder_path
        .as_ref()
        .join(format!("main.{}", source.ext()))
        .display()
        .to_string();

    confirm_and_write(runtime.clone(), &path, content.as_bytes()).await?;
    Ok(())
}
