use std::collections::{BTreeMap, HashSet};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;

use anyhow::{anyhow, Result};
use schemars::schema::{InstanceType, ObjectValidation, RootSchema, Schema, SchemaObject, SingleOrVec};
use schemars::JsonSchema;
use serde_json::{json, Value};
use tailcall::cli::init_file;
use tailcall::{config, FileIO};

static JSON_SCHEMA_FILE: &'static str = "../examples/.tailcallrc.schema.json";
static GRAPHQL_SCHEMA_FILE: &'static str = "../examples/.tailcallrc.graphql";

fn map_type(name: String) -> String {
  match name.as_str() {
    "schema" => "JsonSchema".into(),
    _ => name,
  }
}

struct LineBreaker<'a> {
  string: &'a str,
  break_at: usize,
  index: usize,
}

impl<'a> LineBreaker<'a> {
  fn new(string: &'a str, break_at: usize) -> Self {
    LineBreaker { string, break_at, index: 0 }
  }
}

impl<'a> Iterator for LineBreaker<'a> {
  type Item = &'a str;

  fn next(&mut self) -> Option<Self::Item> {
    if self.index >= self.string.len() {
      return None;
    }

    let end_index = self
      .string
      .chars()
      .skip(self.index + self.break_at)
      .enumerate()
      .find(|(_, ch)| ch.is_whitespace())
      .map(|(index, _)| self.index + self.break_at + index + 1)
      .unwrap_or(self.string.len());

    let start_index = self.index;
    self.index = end_index;

    Some(&self.string[start_index..end_index])
  }
}

struct IndentedWriter<W: Write> {
  writer: W,
  indentation: usize,
  line_broke: bool,
}

impl<W: Write> IndentedWriter<W> {
  fn new(writer: W) -> Self {
    IndentedWriter { writer, indentation: 0, line_broke: false }
  }

  fn indent(&mut self) {
    self.indentation += 2;
  }

  fn unindent(&mut self) {
    self.indentation -= 2;
  }
}

impl<W: std::io::Write> Write for IndentedWriter<W> {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    let mut new_buf = vec![];
    let mut extra = 0;

    for ch in buf {
      if self.line_broke && self.indentation > 0 {
        extra += self.indentation;
        for _ in 0..self.indentation {
          new_buf.push(b' ');
        }
      }
      self.line_broke = false;

      new_buf.push(*ch);
      if ch == &b'\n' {
        self.line_broke = true;
      }
    }

    self.writer.write(&new_buf).map(|a| a - extra)
  }

  fn flush(&mut self) -> std::io::Result<()> {
    self.writer.flush()
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

fn write_description(writer: &mut IndentedWriter<impl Write>, description: Option<&String>) -> std::io::Result<()> {
  if let Some(description) = description {
    let description: String = description.chars().filter(|ch| ch != &'\n').collect();
    let line_breaker = LineBreaker::new(&description, 80);
    writeln!(writer, "\"\"\"")?;
    for line in line_breaker {
      writeln!(writer, "{line}")?;
    }
    writeln!(writer, "\"\"\"")?;
  }
  Ok(())
}

fn write_instance_type(writer: &mut IndentedWriter<impl Write>, typ: &InstanceType) -> std::io::Result<()> {
  match typ {
    &InstanceType::Integer => writeln!(writer, "Int"),
    x => writeln!(writer, "{x:?}"),
  }
}

fn write_reference(writer: &mut IndentedWriter<impl Write>, reference: &String) -> std::io::Result<()> {
  let nm = reference.split("/").last().unwrap();
  match nm {
    "schema" => writeln!(writer, "JsonSchema"),
    other => writeln!(writer, "{other}"),
  }
}

fn write_type(
  writer: &mut IndentedWriter<impl Write>,
  name: String,
  schema: SchemaObject,
  _defs: &BTreeMap<String, Schema>,
  extra_it: &mut BTreeMap<String, ObjectValidation>,
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
      write_instance_type(writer, &typ)
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
      write_instance_type(writer, typ.first().unwrap())
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
                write_reference(writer, &name)
              } else {
                writeln!(writer, "JSON")
              }
            }
            x => write_instance_type(writer, &x),
          }
        } else if let Some(name) = schema.reference.clone() {
          write_reference(writer, &name)
        } else {
          // println!("{name}: {schema:?}");
          writeln!(writer, "JSON")
        }
      } else if let Some(typ) = schema.object.clone() {
        if typ.properties.len() > 0 {
          let upper = name.as_bytes()[0].to_ascii_uppercase();
          let mut name = name;
          unsafe {
            name.as_bytes_mut()[0] = upper;
          }
          writeln!(writer, "{name}")?;
          extra_it.insert(name, *typ);
          Ok(())
        } else {
          writeln!(writer, "JSON")
        }
        // println!("{name}: {schema:?}");
      } else if let Some(sub_schema) = schema.subschemas.clone().into_iter().next() {
        let list = if let Some(list) = sub_schema.any_of {
          list
        } else if let Some(list) = sub_schema.all_of {
          list
        } else if let Some(list) = sub_schema.one_of {
          list
        } else {
          // println!("{name}: {schema:?}");
          writeln!(writer, "JSON")?;
          return Ok(());
        };
        let first = list.first().unwrap();
        match first {
          Schema::Object(obj) => write_reference(writer, &obj.reference.clone().unwrap()),
          _ => panic!(),
        }
      } else if let Some(name) = schema.reference {
        write_reference(writer, &name)
      } else {
        // println!("{name}: {schema:?}");
        writeln!(writer, "JSON")
      }
    }
  }
}

fn write_input_type(
  writer: &mut IndentedWriter<impl Write>,
  mut name: String,
  typ: SchemaObject,
  defs: &BTreeMap<String, Schema>,
  scalars: &mut Vec<String>,
  extra_it: &mut BTreeMap<String, ObjectValidation>,
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
  write_description(writer, description)?;
  if let Some(obj) = typ.object {
    if obj.properties.is_empty() {
      scalars.push(name);
      return Ok(());
    }
    writeln!(writer, "input {name} {{")?;
    writer.indent();
    for (name, property) in obj.properties.into_iter() {
      let property = property.into_object();
      let description = property
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.description.as_ref());
      write_description(writer, description)?;
      write_type(writer, name, property, defs, extra_it)?;
    }
    writer.unindent();
    writeln!(writer, "}}")?;
  } else if let Some(enm) = typ.enum_values {
    writeln!(writer, "enum {name} {{")?;
    writer.indent();
    for val in enm {
      let val: String = format!("{val}").chars().filter(|ch| ch != &'"').collect();
      writeln!(writer, "{val}")?;
    }
    writer.unindent();
    writeln!(writer, "}}")?;
  } else if let Some(list) = typ.subschemas.as_ref().and_then(|ss| ss.any_of.as_ref()) {
    if list.is_empty() {
      scalars.push(name);
      return Ok(());
    }
    writeln!(writer, "input {name} {{")?;
    writer.indent();
    for property in list {
      let property = property.clone().into_object();
      let description = property
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.description.as_ref());
      write_description(writer, description)?;
      if let Some(obj) = property.object {
        for (name, schema) in obj.properties {
          write_type(writer, name, schema.into_object(), defs, extra_it)?;
        }
      }
    }
    writer.unindent();
    writeln!(writer, "}}")?;
  } else if let Some(list) = typ.subschemas.as_ref().and_then(|ss| ss.one_of.as_ref()) {
    if list.is_empty() {
      scalars.push(name);
      return Ok(());
    }
    writeln!(writer, "input {name} {{")?;
    writer.indent();
    for property in list {
      if let Some(obj) = property.clone().into_object().object {
        for (name, schema) in obj.properties {
          write_type(writer, name, schema.into_object(), defs, extra_it)?;
        }
      }
    }
    writer.unindent();
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
  writer: &mut IndentedWriter<impl Write>,
  name: String,
  property: Schema,
  defs: &BTreeMap<String, Schema>,
  extra_it: &mut BTreeMap<String, ObjectValidation>,
) -> std::io::Result<()> {
  // println!("Property: name = {name}");
  let property = property.into_object();
  let description = property
    .metadata
    .as_ref()
    .and_then(|metadata| metadata.description.as_ref());
  write_description(writer, description)?;
  write_type(writer, name, property, defs, extra_it)?;
  Ok(())
}

fn write_schema(
  mut writer: &mut IndentedWriter<impl Write>,
  mut name: String,
  schema: SchemaObject,
  defs: &BTreeMap<String, Schema>,
  on: &str,
  written_directives: &mut HashSet<String>,
  extra_it: &mut BTreeMap<String, ObjectValidation>,
) -> std::io::Result<()> {
  if written_directives.contains(&name) {
    return Ok(());
  }
  // println!("{name}: {:?}", ());
  let description = schema
    .metadata
    .as_ref()
    .and_then(|metadata| metadata.description.as_ref());
  write_description(writer, description)?;
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
      writeln!(writer, "(")?;
      writer.indent();
      write_property(writer, name, property, defs, extra_it)?;
      close_param = true;
    }
    for (name, property) in properties_iter {
      write_property(writer, name, property, defs, extra_it)?;
    }
    if close_param {
      writer.unindent();
      write!(writer, ")")?;
    }
  }
  writeln!(writer, " on {on}\n")?;
  written_directives.insert(name);

  Ok(())
}

fn write_schema_for<T: JsonSchema>(
  writer: &mut IndentedWriter<impl Write>,
  name: &str,
  on: &str,
  written_directives: &mut HashSet<String>,
  extra_it: &mut BTreeMap<String, ObjectValidation>,
) -> Result<()> {
  let schema: RootSchema = schemars::schema_for!(T);
  let defs = schema.definitions;
  write_schema(
    writer,
    name.to_string(),
    schema.schema,
    &defs,
    on,
    written_directives,
    extra_it,
  )?;
  writer.flush()?;
  Ok(())
}

fn write_schema_for_field(
  writer: &mut IndentedWriter<impl Write>,
  written_directives: &mut HashSet<String>,
  extra_it: &mut BTreeMap<String, ObjectValidation>,
) -> Result<()> {
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
      write_schema(
        writer,
        name,
        schema,
        &defs,
        "FIELD_DEFINITION",
        written_directives,
        extra_it,
      )?;
    }
  }

  Ok(())
}

fn write_object_validation(
  writer: &mut IndentedWriter<impl Write>,
  name: String,
  obj_valid: ObjectValidation,
  defs: &BTreeMap<String, Schema>,
  extra_it: &mut BTreeMap<String, ObjectValidation>,
) -> std::io::Result<()> {
  if obj_valid.properties.len() > 0 {
    writeln!(writer, "input {name} {{")?;
    writer.indent();
    for (name, property) in obj_valid.properties {
      write_property(writer, name, property, defs, extra_it)?;
    }
    writer.unindent();
    writeln!(writer, "}}")
  } else {
    Ok(())
  }
}

fn write_all_input_types(
  writer: &mut IndentedWriter<impl Write>,
  mut extra_it: BTreeMap<String, ObjectValidation>,
) -> std::io::Result<()> {
  let schema = schemars::schema_for!(config::Config);

  let defs = schema.definitions;
  let mut scalars = vec![];
  for (name, input_type) in defs.iter() {
    write_input_type(
      writer,
      name.clone(),
      input_type.clone().into_object(),
      &defs,
      &mut scalars,
      &mut extra_it,
    )?;
  }

  let mut new_extra_it = BTreeMap::new();

  for (name, obj_valid) in extra_it.into_iter() {
    write_object_validation(writer, name, obj_valid, &defs, &mut new_extra_it)?;
  }

  for name in scalars {
    writeln!(writer, "scalar {name}")?;
  }

  Ok(())
}

fn generate_rc_file(file: File) -> Result<()> {
  let mut file = IndentedWriter::new(file);
  let mut written_directives = HashSet::new();
  let wd = &mut written_directives;

  let mut extra_it = BTreeMap::new();
  let extra_it_ref = &mut extra_it;

  write_schema_for::<config::Server>(&mut file, "Server", "SCHEMA", wd, extra_it_ref)?;
  write_schema_for::<config::Upstream>(&mut file, "Upstream", "SCHEMA", wd, extra_it_ref)?;
  write_schema_for::<config::AddField>(&mut file, "AddField", "OBJECT", wd, extra_it_ref)?;
  write_schema_for::<config::Cache>(&mut file, "Cache", "OBJECT", wd, extra_it_ref)?;

  write_schema_for_field(&mut file, wd, extra_it_ref)?;

  write_all_input_types(&mut file, extra_it)?;

  writeln!(&mut file, "scalar JSON\n")?;

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
