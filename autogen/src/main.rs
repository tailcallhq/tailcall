use std::collections::{BTreeMap, HashSet};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;

use anyhow::{anyhow, Result};
use schemars::schema::{InstanceType, RootSchema, Schema, SchemaObject, SingleOrVec};
use schemars::JsonSchema;
use serde_json::{json, Value};
use tailcall::cli::init_file;
use tailcall::{config, FileIO};

static JSON_SCHEMA_FILE: &'static str = "../examples/.tailcallrc.schema.json";
static GRAPHQL_SCHEMA_FILE: &'static str = "../examples/.tailcallrc.graphql";

fn map_type(name: String) -> String {
  match name.as_str() {
    "schema" => "JsonSchema".into(),
    _ => name
  }
}

#[tokio::main]
async fn main() {
  logger_init();
  let args: Vec<String> = env::args().collect();
  let arg = args.get(1);

  if arg.is_none() {
    log::error!("An argument required, you can pass either `fix` or `check` argument");
    return;
  }
  match arg.unwrap().as_str() {
    "fix" => {
      let result = mode_fix().await;
      if let Err(e) = result {
        log::error!("{}", e);
        exit(1);
      }
    }
    "check" => {
      let result = mode_check().await;
      if let Err(e) = result {
        log::error!("{}", e);
        exit(1);
      }
    }
    &_ => {
      log::error!("Unknown argument, you can pass either `fix` or `check` argument");
      return;
    }
  }
}

async fn mode_check() -> Result<()> {
  let json_schema = get_file_path();
  let file_io = init_file();
  let content = file_io
    .read(json_schema.to_str().ok_or(anyhow!("Unable to determine path"))?)
    .await?;
  let content = serde_json::from_str::<Value>(&content)?;
  let schema = get_updated_json().await?;
  match content.eq(&schema) {
    true => Ok(()),
    false => Err(anyhow!("Schema mismatch")),
  }
}

async fn mode_fix() -> Result<()> {
  update_json().await?;
  update_gql()?;
  Ok(())
}

async fn update_json() -> Result<()> {
  let path = get_file_path();
  let schema = serde_json::to_string_pretty(&get_updated_json().await?)?;
  let file_io = init_file();
  log::info!("Updating JSON Schema: {}", path.to_str().unwrap());
  file_io
    .write(
      path.to_str().ok_or(anyhow!("Unable to determine path"))?,
      schema.as_bytes(),
    )
    .await?;
  Ok(())
}

fn update_gql() -> Result<()> {
  let file = File::create(GRAPHQL_SCHEMA_FILE)?;
  generate_rc_file(file)?;
  Ok(())
}

fn get_file_path() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(JSON_SCHEMA_FILE)
}

async fn get_updated_json() -> Result<Value> {
  let schema: RootSchema = schemars::schema_for!(config::Server);
  // println!("{schema:#?}");
  let schema = json!(schema);
  Ok(schema)
}

fn write_description(mut writer: impl Write, description: Option<&String>) -> std::io::Result<()> {
  if let Some(description) = description {
    writeln!(writer, "\"\"\"")?;
    writeln!(writer, "{description}")?;
    writeln!(writer, "\"\"\"")?;
  }
  Ok(())
}

fn write_instance_type(mut writer: impl Write, typ: &InstanceType) -> std::io::Result<()> {
  match typ {
    &InstanceType::Integer => writeln!(writer, "Int"),
    x => writeln!(writer, "{x:?}")
  }
}

fn write_reference(mut writer: impl Write, reference: &String) -> std::io::Result<()> {
  let nm = reference.split("/").last().unwrap();
  match nm {
    "schema" => writeln!(writer, "JsonSchema"),
    other => writeln!(writer, "{other}"),
  }
}

fn write_type(
  mut writer: impl Write,
  name: String,
  schema: SchemaObject,
  _defs: &BTreeMap<String, Schema>,
) -> std::io::Result<()> {
  // if name.as_str() == "input" { println!("{name:?}: {schema:?}") };
  write!(writer, "{name}: ")?;
  match schema.instance_type {
    Some(SingleOrVec::Single(typ))
      if matches!(
        *typ,
        InstanceType::Null
          | InstanceType::Boolean
          | InstanceType::Number
          | InstanceType::String
          | InstanceType::Integer
      ) =>
    {
      write_instance_type(&mut writer, &typ)
    }
    Some(SingleOrVec::Vec(typ))
      if matches!(
        typ.first().unwrap(),
        InstanceType::Null
          | InstanceType::Boolean
          | InstanceType::Number
          | InstanceType::String
          | InstanceType::Integer
      ) =>
    {
      write_instance_type(&mut writer, typ.first().unwrap())
    }
    _ => {
      if let Some(schema) = schema.array.clone().and_then(|arr| {
        Some(match arr.items? {
          SingleOrVec::Single(typ) => typ.into_object(),
          SingleOrVec::Vec(typ) => typ.into_iter().next()?.into_object(),
        })
      }) {
        if let Some(it) = schema.instance_type.clone() {
          let typ = match it {
            SingleOrVec::Single(typ) => *typ,
            SingleOrVec::Vec(typ) => typ.into_iter().next().unwrap(),
          };

          match typ {
            InstanceType::Array | InstanceType::Object => {
              if let Some(name) = schema.reference.clone() {
                write_reference(&mut writer, &name)
              } else {
                writeln!(writer, "JSON")
              }
            }
            x => write_instance_type(&mut writer, &x),
          }
        } else if let Some(name) = schema.reference.clone() {
          write_reference(&mut writer, &name)
        } else {
          writeln!(writer, "JSON")
        }
      } else if let Some(_typ) = schema.object.clone() {
        writeln!(writer, "JSON")
      } else if let Some(sub_schema) = schema.subschemas.clone().into_iter().next() {
        let list = if let Some(list) = sub_schema.any_of {
          list
        } else if let Some(list) = sub_schema.all_of {
          list
        } else if let Some(list) = sub_schema.one_of {
          list
        } else {
          writeln!(writer, "JSON")?;
          return Ok(());
        };
        let first = list.first().unwrap();
        match first {
          Schema::Object(obj) => {
            write_reference(&mut writer, &obj.reference.clone().unwrap())
          },
          _ => panic!(),
        }
      } else if let Some(name) = schema.reference {
        write_reference(&mut writer, &name)
      } else {
        // println!("{name}: {schema:?}");
        writeln!(writer, "JSON")
      }
    }
  }
}

fn write_input_type(
  mut writer: impl Write,
  mut name: String,
  typ: SchemaObject,
  defs: &BTreeMap<String, Schema>,
  scalars: &mut Vec<String>
) -> std::io::Result<()> {
  if name.as_str() == "schema" {
    name = "JsonSchema".to_string()
  }

  // println!("InputType {name}");
  // if name.as_str() == "Auth" {
    // println!("{typ:?}");
  // }
  match name.as_str() {
    "Arg" => return Ok(()),
    _ => {}
  }


  let description = typ.metadata.as_ref().and_then(|metadata| metadata.description.as_ref());
  write_description(&mut writer, description)?;
  if let Some(obj) = typ.object {
    if obj.properties.is_empty() {
      scalars.push(name);
      return Ok(());
    }
    writeln!(writer, "input {name} {{")?;
    for (name, property) in obj.properties.into_iter() {
      let property = property.into_object();
      let description = property
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.description.as_ref());
      write_description(&mut writer, description)?;
      write_type(&mut writer, name, property, defs)?;
    }
    writeln!(writer, "}}")?;
  } else if let Some(enm) = typ.enum_values {
    writeln!(writer, "enum {name} {{")?;
    for val in enm {
      let val: String = format!("{val}").chars().filter(|ch| ch != &'"').collect();
      writeln!(writer, "{val}")?;
    }
    writeln!(writer, "}}")?;
  } else if let Some(list) = typ.subschemas.as_ref().and_then(|ss| ss.any_of.as_ref()) {
    if list.is_empty() {
      scalars.push(name);
      return Ok(());
    }
    writeln!(writer, "input {name} {{")?;
    for property in list {
      let property = property.clone().into_object();
      let description = property
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.description.as_ref());
      write_description(&mut writer, description)?;
      if let Some(obj) = property.object {
        for (name, schema) in obj.properties {
          write_type(&mut writer, name, schema.into_object(), defs)?;
        }
      }
    }
    writeln!(writer, "}}")?;
  } else if let Some(list) = typ.subschemas.as_ref().and_then(|ss| ss.one_of.as_ref()) {
    if list.is_empty() {
      scalars.push(name);
      return Ok(());
    }
    writeln!(writer, "input {name} {{")?;
    for property in list {
      if let Some(obj) = property.clone().into_object().object {
        for (name, schema) in obj.properties {
          write_type(&mut writer, name, schema.into_object(), defs)?;
        }
      }
    }
    writeln!(writer, "}}")?;
  } else if let Some(SingleOrVec::Single(item)) = typ.array.and_then(|arr| arr.items) {
    if let Some(name) = item.into_object().reference {
      writeln!(writer, "{name}")?;
    } else {
      scalars.push(name);
    }
  }

  Ok(())
}

fn write_property(
  mut writer: impl Write,
  name: String,
  property: Schema,
  defs: &BTreeMap<String, Schema>,
) -> std::io::Result<()> {
  // println!("Property: name = {name}");
  let property = property.into_object();
  let description = property
    .metadata
    .as_ref()
    .and_then(|metadata| metadata.description.as_ref());
  write_description(&mut writer, description)?;
  write_type(&mut writer, name, property, defs)?;
  Ok(())
}

fn write_schema(
  mut writer: impl Write,
  mut name: String,
  schema: SchemaObject,
  defs: &BTreeMap<String, Schema>,
  on: &str,
  written_directives: &mut HashSet<String>,
) -> std::io::Result<()> {
  if written_directives.contains(&name) { return Ok(()) }
  // println!("{name}: {:?}", ());
  let description = schema
    .metadata
    .as_ref()
    .and_then(|metadata| metadata.description.as_ref());
  write_description(&mut writer, description)?;
  unsafe {
    name.as_bytes_mut().get_mut(0).map(|ch| {
      let lower = (*ch as char).to_ascii_lowercase();
      *ch = lower as u8;
    });
  }
  write!(writer, "directive @{}", name)?;
  if let Some(properties) = schema.object.map(|object| object.properties) {
    let mut properties_iter = properties.into_iter();

    let mut close_param = false;
    if let Some((name, property)) = properties_iter.next() {
      writeln!(writer, " (")?;
      write_property(&mut writer, name, property, defs)?;
      close_param = true;
    }
    for (name, property) in properties_iter {
      write_property(&mut writer, name, property, defs)?;
    }
    if close_param {
      write!(writer, ")")?;
    }
  }
  writeln!(writer, " on {on}")?;
  written_directives.insert(name);

  Ok(())
}

fn write_schema_for<T: JsonSchema>(mut writer: impl Write, name: &str, on: &str, written_directives: &mut HashSet<String>) -> Result<()> {
  let schema: RootSchema = schemars::schema_for!(T);
  let defs = schema.definitions;
  write_schema(&mut writer, name.to_string(), schema.schema, &defs, on, written_directives)?;
  writer.flush()?;
  Ok(())
}

fn write_schema_for_field(mut writer: impl Write, written_directives: &mut HashSet<String>) -> Result<()> {
  let schema = schemars::schema_for!(config::Field);
  // println!("{schema:#?}");
  let defs: BTreeMap<String, Schema> = schema.definitions;
  let defs1: BTreeMap<String, Schema> = defs
    .iter()
    .map(|(k, v)| (k.to_lowercase().clone(), v.clone()))
    .collect();
  for (name, _) in schema.schema.object.unwrap().properties {
    if let Some(schema) = defs1.get(name.as_str()).cloned() {
      let schema = schema.into_object();
      write_schema(&mut writer, name, schema, &defs, "FIELD_DEFINITION", written_directives)?;
    }
  }

  Ok(())
}

fn write_all_input_types(mut writer: impl Write) -> std::io::Result<()> {
  let schema = schemars::schema_for!(config::Config);

  let defs = schema.definitions;
  let mut scalars = vec![];
  for (name, input_type) in defs.iter() {
    write_input_type(&mut writer, name.clone(), input_type.clone().into_object(), &defs, &mut scalars)?;
  }
  // println!("{scalars:?}");
  for name in scalars {
    writeln!(writer, "scalar {name}")?;
  }

  Ok(())
}

fn generate_rc_file(mut file: File) -> Result<()> {
  let mut written_directives = HashSet::new();
  let wd = &mut written_directives;
  write_schema_for::<config::Server>(&mut file, "Server", "SCHEMA", wd)?;
  write_schema_for::<config::Upstream>(&mut file, "Upstream", "SCHEMA", wd)?;
  write_schema_for::<config::AddField>(&mut file, "AddField", "OBJECT", wd)?;
  write_schema_for::<config::Cache>(&mut file, "Cache", "OBJECT", wd)?;

  write_schema_for_field(&mut file, wd)?;

  write_all_input_types(&mut file)?;

  writeln!(&mut file, "scalar JSON")?;

  Ok(())
}

fn logger_init() {
  // set the log level
  const LONG_ENV_FILTER_VAR_NAME: &str = "TAILCALL_SCHEMA_LOG_LEVEL";
  const SHORT_ENV_FILTER_VAR_NAME: &str = "TC_SCHEMA_LOG_LEVEL";

  // Select which env variable to use for the log level filter. This is because filter_or doesn't allow picking between multiple env_var for the filter value
  let filter_env_name = env::var(LONG_ENV_FILTER_VAR_NAME)
    .map(|_| LONG_ENV_FILTER_VAR_NAME)
    .unwrap_or_else(|_| SHORT_ENV_FILTER_VAR_NAME);

  // use the log level from the env if there is one, otherwise use the default.
  let env = env_logger::Env::new().filter_or(filter_env_name, "info");

  env_logger::Builder::from_env(env).init();
}
