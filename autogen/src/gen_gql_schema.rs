use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::Write;

use anyhow::Result;
use schemars::schema::{
    ArrayValidation, InstanceType, ObjectValidation, Schema, SchemaObject, SingleOrVec,
};
use tailcall::config;

static GRAPHQL_SCHEMA_FILE: &str = "generated/.tailcallrc.graphql";
static DIRECTIVE_ALLOW_LIST: [(&str, Entity, bool); 13] = [
    ("server", Entity::Schema, false),
    ("link", Entity::Schema, true),
    ("upstream", Entity::Schema, false),
    ("http", Entity::FieldDefinition, false),
    ("grpc", Entity::FieldDefinition, false),
    ("addField", Entity::Object, true),
    ("modify", Entity::FieldDefinition, false),
    ("groupBy", Entity::FieldDefinition, false),
    ("const", Entity::FieldDefinition, false),
    ("graphQL", Entity::FieldDefinition, false),
    ("cache", Entity::FieldDefinition, false),
    ("expr", Entity::FieldDefinition, false),
    ("js", Entity::FieldDefinition, false),
];
static OBJECT_WHITELIST: [&str; 18] = [
    "ExprBody",
    "If",
    "Http",
    "Grpc",
    "GraphQL",
    "Proxy",
    "KeyValue",
    "Batch",
    "HttpVersion",
    "Method",
    "Encoding",
    "Cache",
    "Const",
    "Encoding",
    "Expr",
    "ExprBody",
    "JS",
    "Modify",
];

#[derive(Clone, Copy)]
enum Entity {
    Schema,
    Object,
    FieldDefinition,
}

impl std::fmt::Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Entity::Schema => {
                write!(f, "SCHEMA")
            }
            Entity::Object => {
                write!(f, "OBJECT")
            }
            Entity::FieldDefinition => {
                write!(f, "FIELD_DEFINITION")
            }
        }
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
        let mut new_buf = Vec::with_capacity(
            buf.len() + self.indentation * buf.iter().filter(|&&c| c == b'\n').count(),
        );
        let mut extra = 0;

        for ch in buf {
            if self.line_broke && self.indentation > 0 {
                extra += self.indentation;
                new_buf.extend((0..self.indentation).map(|_| b' '));
            }
            self.line_broke = *ch == b'\n';

            new_buf.push(*ch);
        }

        self.writer.write(&new_buf).map(|a| a - extra)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

#[derive(Debug)]
enum ExtraTypes {
    Schema,
    ObjectValidation(ObjectValidation),
}

fn write_description(
    writer: &mut IndentedWriter<impl Write>,
    description: Option<&String>,
) -> std::io::Result<()> {
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

fn write_instance_type(
    writer: &mut IndentedWriter<impl Write>,
    typ: &InstanceType,
) -> std::io::Result<()> {
    match typ {
        &InstanceType::Integer => write!(writer, "Int"),
        x => write!(writer, "{x:?}"),
    }
}

fn write_reference(
    writer: &mut IndentedWriter<impl Write>,
    reference: &str,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
) -> std::io::Result<()> {
    let mut nm = reference.split('/').last().unwrap().to_string();
    first_char_to_upper(&mut nm);
    extra_it.insert(nm.clone(), ExtraTypes::Schema);
    write!(writer, "{nm}")
}

fn first_char_to_upper(name: &mut String) {
    if let Some(first_char) = name.chars().next() {
        // Remove the first character and make it uppercase
        let first_char_upper = first_char.to_uppercase().to_string();

        // Remove the first character from the original string
        let mut chars = name.chars();
        chars.next();

        // Replace the original string with the new one
        *name = first_char_upper + chars.as_str();
    }
}

struct WriteParams<'a, W: Write> {
    writer: &'a mut IndentedWriter<W>,
    name: String,
    schema: SchemaObject,
    defs: &'a BTreeMap<String, Schema>,
    scalars: &'a mut HashSet<String>,
    extra_it: &'a mut BTreeMap<String, ExtraTypes>,
    arr_valid: ArrayValidation,
    obj_valid: ObjectValidation,
}

fn write_type<W: Write>(params: WriteParams<'_, W>) -> std::io::Result<()> {
    let WriteParams { writer, name, schema, defs, extra_it, .. } = params;
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
            write_instance_type(writer, &typ)?;
            write!(writer, "!")
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
            if let Some(arr_valid) = schema.array.clone() {
                let params = WriteParams {
                    writer,
                    name,
                    schema: SchemaObject::default(),
                    defs,
                    scalars: &mut HashSet::new(),
                    extra_it,
                    arr_valid: *arr_valid,
                    obj_valid: ObjectValidation::default(),
                };
                //write_array_validation(writer, name, *arr_valid, defs, extra_it)
                write_array_validation(params)
            } else if let Some(typ) = schema.object.clone() {
                if !typ.properties.is_empty() {
                    let mut name = name;
                    first_char_to_upper(&mut name);
                    write!(writer, "{name}")?;
                    extra_it.insert(name, ExtraTypes::ObjectValidation(*typ));
                    Ok(())
                } else {
                    write!(writer, "JSON")
                }
            } else if let Some(sub_schema) = schema.subschemas.clone().into_iter().next() {
                let list = if let Some(list) = sub_schema.any_of {
                    list
                } else if let Some(list) = sub_schema.all_of {
                    list
                } else if let Some(list) = sub_schema.one_of {
                    list
                } else {
                    write!(writer, "JSON")?;
                    return Ok(());
                };
                let first = list.first().unwrap();
                match first {
                    Schema::Object(obj) => {
                        write_reference(writer, &obj.reference.clone().unwrap(), extra_it)
                    }
                    _ => panic!(),
                }
            } else if let Some(name) = schema.reference {
                write_reference(writer, &name, extra_it)
            } else {
                write!(writer, "JSON")
            }
        }
    }
}

fn write_field<W: Write>(params: WriteParams<'_, W>) -> std::io::Result<()> {
    let WriteParams { writer, name, .. } = params;
    write!(writer, "{name}: ")?;
    let params_clone = WriteParams { writer, name: name.clone(), ..params };
    write_type(params_clone)?;
    writeln!(writer)
}

fn write_input_type<W: Write>(params: WriteParams<'_, W>) -> std::io::Result<()> {
    let WriteParams { writer, name, schema: typ, defs, scalars, extra_it, .. } = params;

    let name = match input_whitelist_lookup(&name, extra_it) {
        Some(name) => name,
        None => return Ok(()),
    };
    let description = typ
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.description.as_ref());
    write_description(writer, description)?;
    if let Some(obj) = typ.object {
        if obj.properties.is_empty() {
            scalars.insert(name.to_string());
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
            let params = WriteParams {
                writer,
                name,
                schema: property,
                defs,
                scalars,
                extra_it,
                arr_valid: ArrayValidation::default(),
                obj_valid: ObjectValidation::default(),
            };
            write_field(params)?;
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
            scalars.insert(name.to_string());
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
                    let params = WriteParams {
                        writer,
                        name,
                        schema: schema.into_object(),
                        defs,
                        scalars,
                        extra_it,
                        arr_valid: ArrayValidation::default(),
                        obj_valid: ObjectValidation::default(),
                    };
                    write_field(params)?;
                }
            }
        }
        writer.unindent();
        writeln!(writer, "}}")?;
    } else if let Some(list) = typ.subschemas.as_ref().and_then(|ss| ss.one_of.as_ref()) {
        if list.is_empty() {
            scalars.insert(name.to_string());
            return Ok(());
        }
        writeln!(writer, "input {name} {{")?;
        writer.indent();
        for property in list {
            if let Some(obj) = property.clone().into_object().object {
                for (name, schema) in obj.properties {
                    let params = WriteParams {
                        writer,
                        name,
                        schema: schema.into_object(),
                        defs,
                        scalars,
                        extra_it,
                        arr_valid: ArrayValidation::default(),
                        obj_valid: ObjectValidation::default(),
                    };
                    write_field(params)?;
                }
            }
        }
        writer.unindent();
        writeln!(writer, "}}")?;
    } else if let Some(SingleOrVec::Single(item)) = typ.array.and_then(|arr| arr.items) {
        if let Some(name) = item.into_object().reference {
            writeln!(writer, "{name}")?;
        } else {
            scalars.insert(name.to_string());
        }
    }

    Ok(())
}

fn write_property<W: Write>(params: WriteParams<'_, W>) -> std::io::Result<()> {
    let WriteParams { writer, name, schema: property, defs, extra_it, .. } = params;
    //let property = property.into_object();
    let description = property
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.description.as_ref());
    write_description(writer, description)?;
    let params = WriteParams {
        writer,
        name,
        schema: property,
        defs,
        scalars: &mut HashSet::new(),
        extra_it,
        arr_valid: ArrayValidation::default(),
        obj_valid: ObjectValidation::default(),
    };
    write_field(params)?;
    Ok(())
}

fn directive_allow_list_lookup(name: &str) -> Option<(&'static str, Entity, bool)> {
    for (nm, entity, is_repeatable) in DIRECTIVE_ALLOW_LIST.iter() {
        if name.to_lowercase() == nm.to_lowercase() {
            return Some((*nm, *entity, *is_repeatable));
        }
    }
    None
}

fn input_whitelist_lookup<'a>(
    name: &'a str,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
) -> Option<&'a str> {
    for nm in OBJECT_WHITELIST.iter() {
        if name.to_lowercase() == nm.to_lowercase() {
            return Some(*nm);
        }
    }

    if extra_it.contains_key(name) {
        return Some(name);
    }

    None
}

fn write_directive<W: Write>(params: WriteParams<'_, W>) -> std::io::Result<()> {
    let WriteParams {
        writer,
        name,
        schema,
        defs,
        scalars: written_directives,
        extra_it,
        ..
    } = params;
    let (name, entity, is_repeatable) = match directive_allow_list_lookup(&name) {
        Some(entity) => entity,
        None => return Ok(()),
    };

    if written_directives.contains(name) {
        return Ok(());
    }

    let description = schema
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.description.as_ref());
    write_description(writer, description)?;

    write!(writer, "directive @{}", name)?;
    if let Some(properties) = schema.object.map(|object| object.properties) {
        let mut properties_iter = properties.into_iter();

        let mut close_param = false;
        if let Some((name, property)) = properties_iter.next() {
            writeln!(writer, "(")?;
            writer.indent();
            let params = WriteParams {
                writer,
                name,
                schema: property.into_object(),
                defs,
                scalars: &mut HashSet::new(),
                extra_it,
                arr_valid: ArrayValidation::default(),
                obj_valid: ObjectValidation::default(),
            };
            write_property(params)?;
            close_param = true;
        }
        for (name, property) in properties_iter {
            let params = WriteParams {
                writer,
                name,
                schema: property.into_object(),
                defs,
                scalars: &mut HashSet::new(),
                extra_it,
                arr_valid: ArrayValidation::default(),
                obj_valid: ObjectValidation::default(),
            };
            write_property(params)?;
        }
        if close_param {
            writer.unindent();
            write!(writer, ")")?;
        }
    }

    if is_repeatable {
        write!(writer, " repeatable ")?;
    }

    writeln!(writer, " on {entity:?}\n")?;
    written_directives.insert(name.to_string());

    Ok(())
}

fn write_all_directives(
    writer: &mut IndentedWriter<impl Write>,
    written_directives: &mut HashSet<String>,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
) -> Result<()> {
    let schema = schemars::schema_for!(config::Config);

    let defs: BTreeMap<String, Schema> = schema.definitions;
    let dirs: BTreeMap<String, Schema> = defs.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    for (name, schema) in dirs.into_iter() {
        let schema = schema.clone().into_object();
        let params = WriteParams {
            writer,
            name: name.clone(),
            schema,
            defs: &defs,
            scalars: written_directives,
            extra_it,
            arr_valid: ArrayValidation::default(),
            obj_valid: ObjectValidation::default(),
        };
        write_directive(params)?;
    }

    Ok(())
}

fn write_array_validation<W: Write>(params: WriteParams<'_, W>) -> std::io::Result<()> {
    let WriteParams { writer, name, defs, extra_it, arr_valid, .. } = params;
    write!(writer, "[")?;
    if let Some(SingleOrVec::Single(schema)) = arr_valid.items {
        let params = WriteParams {
            writer,
            name,
            schema: schema.into_object(),
            defs,
            scalars: &mut HashSet::new(),
            extra_it,
            arr_valid: ArrayValidation::default(),
            obj_valid: ObjectValidation::default(),
        };
        write_type(params)?;
    } else if let Some(SingleOrVec::Vec(schemas)) = arr_valid.items {
        let params = WriteParams {
            writer,
            name,
            schema: schemas[0].clone().into_object(),
            defs,
            scalars: &mut HashSet::new(),
            extra_it,
            arr_valid: ArrayValidation::default(),
            obj_valid: ObjectValidation::default(),
        };
        write_type(params)?;
    } else {
        println!("{name}: {arr_valid:?}");

        write!(writer, "JSON")?;
    }
    write!(writer, "]")
}

fn write_object_validation<W: Write>(params: WriteParams<'_, W>) -> std::io::Result<()> {
    let WriteParams { writer, name, defs, extra_it, obj_valid, .. } = params;
    if !obj_valid.properties.is_empty() {
        writeln!(writer, "input {name} {{")?;
        writer.indent();
        for (name, property) in obj_valid.properties {
            let params = WriteParams {
                writer,
                name,
                schema: property.into_object(),
                defs,
                scalars: &mut HashSet::new(),
                extra_it,
                arr_valid: ArrayValidation::default(),
                obj_valid: ObjectValidation::default(),
            };
            write_property(params)?;
        }
        writer.unindent();
        writeln!(writer, "}}")
    } else {
        Ok(())
    }
}

fn write_all_input_types(
    writer: &mut IndentedWriter<impl Write>,
    mut extra_it: BTreeMap<String, ExtraTypes>,
) -> std::io::Result<()> {
    let schema = schemars::schema_for!(config::Config);

    let defs = schema.definitions;
    let mut scalars = HashSet::new();
    for (name, input_type) in defs.iter() {
        let mut name = name.clone();
        first_char_to_upper(&mut name);
        let params = WriteParams {
            writer,
            name,
            schema: input_type.clone().into_object(),
            defs: &defs,
            scalars: &mut scalars,
            extra_it: &mut extra_it,
            arr_valid: ArrayValidation::default(),
            obj_valid: ObjectValidation::default(),
        };
        write_input_type(params)?;
    }

    let mut new_extra_it = BTreeMap::new();

    for (name, extra_type) in extra_it.into_iter() {
        match extra_type {
            ExtraTypes::Schema => {
                if let Some(schema) = defs.get(&name).cloned() {
                    let params = WriteParams {
                        writer,
                        name,
                        schema: schema.into_object(),
                        defs: &defs,
                        scalars: &mut scalars,
                        extra_it: &mut new_extra_it,
                        arr_valid: ArrayValidation::default(),
                        obj_valid: ObjectValidation::default(),
                    };
                    write_input_type(params)?
                }
            }
            ExtraTypes::ObjectValidation(obj_valid) => {
                //write_object_validation(writer, name, obj_valid, &defs, &mut new_extra_it)?
                let params = WriteParams {
                    writer,
                    name,
                    schema: SchemaObject::default(),
                    defs: &defs,
                    scalars: &mut scalars,
                    extra_it: &mut new_extra_it,
                    arr_valid: ArrayValidation::default(),
                    obj_valid,
                };
                write_object_validation(params)?;
            }
        }
    }

    for name in scalars {
        writeln!(writer, "scalar {name}")?;
    }

    Ok(())
}

pub fn update_gql() -> Result<()> {
    let file = File::create(GRAPHQL_SCHEMA_FILE)?;
    generate_rc_file(file)?;
    Ok(())
}

fn generate_rc_file(file: File) -> Result<()> {
    let mut file = IndentedWriter::new(file);
    let mut written_directives = HashSet::new();

    let mut extra_it = BTreeMap::new();

    write_all_directives(&mut file, &mut written_directives, &mut extra_it)?;
    write_all_input_types(&mut file, extra_it)?;

    writeln!(&mut file, "scalar JSON\n")?;

    Ok(())
}
