use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::{self, Display};
use std::num::NonZeroU64;

use anyhow::Result;
use async_graphql::parser::types::ServiceDocument;
use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::telemetry::Telemetry;
use super::{Expr, KeyValue, Link, Server, Upstream};
use crate::config::from_document::from_document;
use crate::config::source::Source;
use crate::directive::DirectiveCodec;
use crate::http::Method;
use crate::json::JsonSchema;
use crate::valid::{Valid, Validator};
use crate::{is_default, scalar};

#[derive(
    Serialize, Deserialize, Clone, Debug, Default, Setters, PartialEq, Eq, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    ///
    /// Dictates how the server behaves and helps tune tailcall for all ingress
    /// requests. Features such as request batching, SSL, HTTP2 etc. can be
    /// configured here.
    #[serde(default)]
    pub server: Server,

    ///
    /// Dictates how tailcall should handle upstream requests/responses.
    /// Tuning upstream can improve performance and reliability for connections.
    #[serde(default)]
    pub upstream: Upstream,

    ///
    /// Specifies the entry points for query and mutation in the generated
    /// GraphQL schema.
    pub schema: RootSchema,

    ///
    /// A map of all the types in the schema.
    #[serde(default)]
    #[setters(skip)]
    pub types: BTreeMap<String, Type>,

    ///
    /// A map of all the union types in the schema.
    #[serde(default, skip_serializing_if = "is_default")]
    pub unions: BTreeMap<String, Union>,

    ///
    /// A list of all links in the schema.
    #[serde(default, skip_serializing_if = "is_default")]
    pub links: Vec<Link>,
    #[serde(default, skip_serializing_if = "is_default")]
    /// Enable [opentelemetry](https://opentelemetry.io) support
    pub opentelemetry: Telemetry,
}

impl Config {
    pub fn port(&self) -> u16 {
        self.server.port.unwrap_or(8000)
    }

    pub fn output_types(&self) -> HashSet<&String> {
        let mut types = HashSet::new();
        let input_types = self.input_types();

        if let Some(ref query) = &self.schema.query {
            types.insert(query);
        }

        if let Some(ref mutation) = &self.schema.mutation {
            types.insert(mutation);
        }
        for (type_name, type_of) in self.types.iter() {
            if (type_of.interface || !type_of.fields.is_empty())
                && !input_types.contains(&type_name)
            {
                for (_, field) in type_of.fields.iter() {
                    types.insert(&field.type_of);
                }
            }
        }
        types
    }

    pub fn recurse_type<'a>(&'a self, type_of: &str, types: &mut HashSet<&'a String>) {
        if let Some(type_) = self.find_type(type_of) {
            for (_, field) in type_.fields.iter() {
                if !types.contains(&field.type_of) {
                    types.insert(&field.type_of);
                    self.recurse_type(&field.type_of, types);
                }
            }
        }
    }

    pub fn input_types(&self) -> HashSet<&String> {
        let mut types = HashSet::new();
        for (_, type_of) in self.types.iter() {
            if !type_of.interface {
                for (_, field) in type_of.fields.iter() {
                    for (_, arg) in field
                        .args
                        .iter()
                        .filter(|(_, arg)| !scalar::is_scalar(&arg.type_of))
                    {
                        if let Some(t) = self.find_type(&arg.type_of) {
                            t.fields.iter().for_each(|(_, f)| {
                                types.insert(&f.type_of);
                                self.recurse_type(&f.type_of, &mut types)
                            })
                        }
                        types.insert(&arg.type_of);
                    }
                }
            }
        }
        types
    }
    pub fn find_type(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }

    pub fn find_union(&self, name: &str) -> Option<&Union> {
        self.unions.get(name)
    }

    pub fn to_yaml(&self) -> Result<String> {
        Ok(serde_yaml::to_string(self)?)
    }

    pub fn to_json(&self, pretty: bool) -> Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            Ok(serde_json::to_string(self)?)
        }
    }

    pub fn to_document(&self) -> ServiceDocument {
        self.clone().into()
    }

    pub fn to_sdl(&self) -> String {
        let doc = self.to_document();
        crate::document::print(doc)
    }

    pub fn query(mut self, query: &str) -> Self {
        self.schema.query = Some(query.to_string());
        self
    }

    pub fn types(mut self, types: Vec<(&str, Type)>) -> Self {
        let mut graphql_types = BTreeMap::new();
        for (name, type_) in types {
            graphql_types.insert(name.to_string(), type_);
        }
        self.types = graphql_types;
        self
    }

    pub fn contains(&self, name: &str) -> bool {
        self.types.contains_key(name) || self.unions.contains_key(name)
    }

    pub fn merge_right(self, other: &Self) -> Self {
        let server = self.server.merge_right(other.server.clone());
        let types = merge_types(self.types, other.types.clone());
        let unions = merge_unions(self.unions, other.unions.clone());
        let schema = self.schema.merge_right(other.schema.clone());
        let upstream = self.upstream.merge_right(other.upstream.clone());
        let links = merge_links(self.links, other.links.clone());
        let opentelemetry = self.opentelemetry.merge_right(other.opentelemetry.clone());

        Self {
            server,
            upstream,
            types,
            schema,
            unions,
            links,
            opentelemetry,
        }
    }
}

fn merge_links(self_links: Vec<Link>, other_links: Vec<Link>) -> Vec<Link> {
    let mut links = self_links.clone();
    let other_links = other_links.clone();

    links.extend(other_links);

    links
}

///
/// Represents a GraphQL type.
/// A type can be an object, interface, enum or scalar.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
pub struct Type {
    ///
    /// A map of field name and its definition.
    pub fields: BTreeMap<String, Field>,
    #[serde(default, skip_serializing_if = "is_default")]
    ///
    /// Additional fields to be added to the type
    pub added_fields: Vec<AddField>,
    #[serde(default, skip_serializing_if = "is_default")]
    ///
    /// Documentation for the type that is publicly visible.
    pub doc: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    ///
    /// Flag to indicate if the type is an interface.
    pub interface: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    ///
    /// Interfaces that the type implements.
    pub implements: BTreeSet<String>,
    #[serde(rename = "enum", default, skip_serializing_if = "is_default")]
    ///
    /// Variants for the type if it's an enum
    pub variants: Option<BTreeSet<String>>,
    #[serde(default, skip_serializing_if = "is_default")]
    ///
    /// Flag to indicate if the type is a scalar.
    pub scalar: bool,
    #[serde(default)]
    ///
    /// Setting to indicate if the type can be cached.
    pub cache: Option<Cache>,
}

impl Type {
    pub fn fields(mut self, fields: Vec<(&str, Field)>) -> Self {
        let mut graphql_fields = BTreeMap::new();
        for (name, field) in fields {
            graphql_fields.insert(name.to_string(), field);
        }
        self.fields = graphql_fields;
        self
    }
    pub fn merge_right(mut self, other: &Self) -> Self {
        let mut fields = self.fields.clone();
        fields.extend(other.fields.clone());
        self.implements.extend(other.implements.clone());
        if let Some(ref variants) = self.variants {
            if let Some(ref other) = other.variants {
                self.variants = Some(variants.union(other).cloned().collect());
            }
        } else {
            self.variants = other.variants.clone();
        }
        Self { fields, ..self.clone() }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Eq, schemars::JsonSchema)]
/// The @cache operator enables caching for the query, field or type it is
/// applied to.
#[serde(rename_all = "camelCase")]
pub struct Cache {
    /// Specifies the duration, in milliseconds, of how long the value has to be
    /// stored in the cache.
    pub max_age: NonZeroU64,
}

fn merge_types(
    mut self_types: BTreeMap<String, Type>,
    other_types: BTreeMap<String, Type>,
) -> BTreeMap<String, Type> {
    for (name, mut other_type) in other_types {
        if let Some(self_type) = self_types.remove(&name) {
            other_type = self_type.merge_right(&other_type);
        }

        self_types.insert(name, other_type);
    }
    self_types
}

fn merge_unions(
    mut self_unions: BTreeMap<String, Union>,
    other_unions: BTreeMap<String, Union>,
) -> BTreeMap<String, Union> {
    for (name, mut other_union) in other_unions {
        if let Some(self_union) = self_unions.remove(&name) {
            other_union = self_union.merge_right(other_union);
        }
        self_unions.insert(name, other_union);
    }
    self_unions
}

#[derive(
    Serialize, Deserialize, Clone, Debug, Default, Setters, PartialEq, Eq, schemars::JsonSchema,
)]
#[setters(strip_option)]
pub struct RootSchema {
    pub query: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub mutation: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub subscription: Option<String>,
}

impl RootSchema {
    // TODO: add unit-tests
    fn merge_right(self, other: Self) -> Self {
        Self {
            query: other.query.or(self.query),
            mutation: other.mutation.or(self.mutation),
            subscription: other.subscription.or(self.subscription),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
/// Used to omit a field from public consumption.
pub struct Omit {}

///
/// A field definition containing all the metadata information about resolving a
/// field.
#[derive(
    Serialize, Deserialize, Clone, Debug, Default, Setters, PartialEq, Eq, schemars::JsonSchema,
)]
#[setters(strip_option)]
pub struct Field {
    ///
    /// Refers to the type of the value the field can be resolved to.
    #[serde(rename = "type", default, skip_serializing_if = "is_default")]
    pub type_of: String,

    ///
    /// Flag to indicate the type is a list.
    #[serde(default, skip_serializing_if = "is_default")]
    pub list: bool,

    ///
    /// Flag to indicate the type is required.
    #[serde(default, skip_serializing_if = "is_default")]
    pub required: bool,

    ///
    /// Flag to indicate if the type inside the list is required.
    #[serde(default, skip_serializing_if = "is_default")]
    pub list_type_required: bool,

    ///
    /// Map of argument name and its definition.
    #[serde(default, skip_serializing_if = "is_default")]
    pub args: BTreeMap<String, Arg>,

    ///
    /// Publicly visible documentation for the field.
    #[serde(default, skip_serializing_if = "is_default")]
    pub doc: Option<String>,

    ///
    /// Allows modifying existing fields.
    #[serde(default, skip_serializing_if = "is_default")]
    pub modify: Option<Modify>,

    ///
    /// Omits a field from public consumption.
    #[serde(default, skip_serializing_if = "is_default")]
    pub omit: Option<Omit>,

    ///
    /// Inserts an HTTP resolver for the field.
    #[serde(default, skip_serializing_if = "is_default")]
    pub http: Option<Http>,

    ///
    /// Inserts a call resolver for the field.
    #[serde(default, skip_serializing_if = "is_default")]
    pub call: Option<Call>,

    ///
    /// Inserts a GRPC resolver for the field.
    #[serde(default, skip_serializing_if = "is_default")]
    pub grpc: Option<Grpc>,

    ///
    /// Inserts a Javascript resolver for the field.
    #[serde(default, skip_serializing_if = "is_default")]
    pub script: Option<JS>,

    ///
    /// Inserts a constant resolver for the field.
    #[serde(rename = "const", default, skip_serializing_if = "is_default")]
    pub const_field: Option<Const>,

    ///
    /// Inserts a GraphQL resolver for the field.
    #[serde(default, skip_serializing_if = "is_default")]
    pub graphql: Option<GraphQL>,

    ///
    /// Inserts an Expression resolver for the field.
    #[serde(default, skip_serializing_if = "is_default")]
    pub expr: Option<Expr>,
    ///
    /// Sets the cache configuration for a field
    pub cache: Option<Cache>,
}

impl Field {
    pub fn has_resolver(&self) -> bool {
        self.http.is_some()
            || self.script.is_some()
            || self.const_field.is_some()
            || self.graphql.is_some()
            || self.grpc.is_some()
            || self.expr.is_some()
            || self.call.is_some()
    }

    /// Returns a list of resolvable directives for the field.
    pub fn resolvable_directives(&self) -> Vec<String> {
        let mut directives = Vec::new();
        if self.http.is_some() {
            directives.push(Http::trace_name());
        }
        if self.graphql.is_some() {
            directives.push(GraphQL::trace_name());
        }
        if self.script.is_some() {
            directives.push(JS::trace_name());
        }
        if self.const_field.is_some() {
            directives.push(Const::trace_name());
        }
        if self.grpc.is_some() {
            directives.push(Grpc::trace_name());
        }
        if self.call.is_some() {
            directives.push(Call::trace_name());
        }
        directives
    }
    pub fn has_batched_resolver(&self) -> bool {
        self.http
            .as_ref()
            .is_some_and(|http| !http.group_by.is_empty())
            || self.graphql.as_ref().is_some_and(|graphql| graphql.batch)
            || self
                .grpc
                .as_ref()
                .is_some_and(|grpc| !grpc.group_by.is_empty())
    }
    pub fn to_list(mut self) -> Self {
        self.list = true;
        self
    }

    pub fn int() -> Self {
        Self { type_of: "Int".to_string(), ..Default::default() }
    }

    pub fn string() -> Self {
        Self { type_of: "String".to_string(), ..Default::default() }
    }

    pub fn float() -> Self {
        Self { type_of: "Float".to_string(), ..Default::default() }
    }

    pub fn boolean() -> Self {
        Self { type_of: "Boolean".to_string(), ..Default::default() }
    }

    pub fn id() -> Self {
        Self { type_of: "ID".to_string(), ..Default::default() }
    }

    pub fn is_omitted(&self) -> bool {
        self.omit.is_some() || self.modify.as_ref().map(|m| m.omit).is_some()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
pub struct JS {
    pub script: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
pub struct Modify {
    #[serde(default, skip_serializing_if = "is_default")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub omit: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Inline {
    pub path: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
pub struct Arg {
    #[serde(rename = "type")]
    pub type_of: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub list: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub required: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub doc: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub modify: Option<Modify>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub default_value: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
pub struct Union {
    pub types: BTreeSet<String>,
    pub doc: Option<String>,
}

impl Union {
    pub fn merge_right(mut self, other: Self) -> Self {
        self.types.extend(other.types);
        self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
/// The @http operator indicates that a field or node is backed by a REST API.
///
/// For instance, if you add the @http operator to the `users` field of the
/// Query type with a path argument of `"/users"`, it signifies that the `users`
/// field is backed by a REST API. The path argument specifies the path of the
/// REST API. In this scenario, the GraphQL server will make a GET request to
/// the API endpoint specified when the `users` field is queried.
pub struct Http {
    #[serde(rename = "baseURL", default, skip_serializing_if = "is_default")]
    /// This refers to the base URL of the API. If not specified, the default
    /// base URL is the one specified in the `@upstream` operator.
    pub base_url: Option<String>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The body of the API call. It's used for methods like POST or PUT that
    /// send data to the server. You can pass it as a static object or use a
    /// Mustache template to substitute variables from the GraphQL variables.
    pub body: Option<String>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The `encoding` parameter specifies the encoding of the request body. It
    /// can be `ApplicationJson` or `ApplicationXWwwFormUrlEncoded`. @default
    /// `ApplicationJson`.
    pub encoding: Encoding,

    #[serde(rename = "batchKey", default, skip_serializing_if = "is_default")]
    /// The `batchKey` parameter groups multiple data requests into a single call. For more details please refer out [n + 1 guide](https://tailcall.run/docs/guides/n+1#solving-using-batching).
    pub group_by: Vec<String>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The `headers` parameter allows you to customize the headers of the HTTP
    /// request made by the `@http` operator. It is used by specifying a
    /// key-value map of header names and their values.
    pub headers: Vec<KeyValue>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Schema of the input of the API call. It is automatically inferred in
    /// most cases.
    pub input: Option<JsonSchema>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// This refers to the HTTP method of the API call. Commonly used methods
    /// include `GET`, `POST`, `PUT`, `DELETE` etc. @default `GET`.
    pub method: Method,

    /// This refers to the API endpoint you're going to call. For instance `https://jsonplaceholder.typicode.com/users`.
    ///
    /// For dynamic segments in your API endpoint, use Mustache templates for
    /// variable substitution. For instance, to fetch a specific user, use
    /// `/users/{{args.id}}`.
    pub path: String,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Schema of the output of the API call. It is automatically inferred in
    /// most cases.
    pub output: Option<JsonSchema>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// This represents the query parameters of your API call. You can pass it
    /// as a static object or use Mustache template for dynamic parameters.
    /// These parameters will be added to the URL.
    pub query: Vec<KeyValue>,
}

///
/// Provides the ability to refer to a field defined in the root Query or
/// Mutation.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
pub struct Call {
    #[serde(default, skip_serializing_if = "is_default")]
    /// The name of the field on the `Query` type that you want to call. For
    /// instance `user`.
    pub query: Option<String>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The name of the field on the `Mutation` type that you want to call. For
    /// instance `createUser`.
    pub mutation: Option<String>,

    /// The arguments of the field on the `Query` or `Mutation` type that you
    /// want to call. For instance `{id: "{{value.userId}}"}`.
    #[serde(default, skip_serializing_if = "is_default")]
    pub args: BTreeMap<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
/// The @grpc operator indicates that a field or node is backed by a gRPC API.
///
/// For instance, if you add the @grpc operator to the `users` field of the
/// Query type with a service argument of `NewsService` and method argument of
/// `GetAllNews`, it signifies that the `users` field is backed by a gRPC API.
/// The `service` argument specifies the name of the gRPC service.
/// The `method` argument specifies the name of the gRPC method.
/// In this scenario, the GraphQL server will make a gRPC request to the gRPC
/// endpoint specified when the `users` field is queried.
pub struct Grpc {
    #[serde(rename = "baseURL", default, skip_serializing_if = "is_default")]
    /// This refers to the base URL of the API. If not specified, the default
    /// base URL is the one specified in the `@upstream` operator.
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    /// This refers to the arguments of your gRPC call. You can pass it as a
    /// static object or use Mustache template for dynamic parameters. These
    /// parameters will be added in the body in `protobuf` format.
    pub body: Option<String>,
    #[serde(rename = "batchKey", default, skip_serializing_if = "is_default")]
    /// The key path in the response which should be used to group multiple requests. For instance `["news","id"]`. For more details please refer out [n + 1 guide](https://tailcall.run/docs/guides/n+1#solving-using-batching).
    pub group_by: Vec<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    /// The `headers` parameter allows you to customize the headers of the HTTP
    /// request made by the `@grpc` operator. It is used by specifying a
    /// key-value map of header names and their values. Note: content-type is
    /// automatically set to application/grpc
    pub headers: Vec<KeyValue>,
    /// This refers to the gRPC method you're going to call. For instance
    /// `GetAllNews`.
    pub method: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
/// The @graphQL operator allows to specify GraphQL API server request to fetch
/// data from.
pub struct GraphQL {
    #[serde(default, skip_serializing_if = "is_default")]
    /// Named arguments for the requested field. More info [here](https://tailcall.run/docs/guides/operators/#args)
    pub args: Option<Vec<KeyValue>>,

    #[serde(rename = "baseURL", default, skip_serializing_if = "is_default")]
    /// This refers to the base URL of the API. If not specified, the default
    /// base URL is the one specified in the `@upstream` operator.
    pub base_url: Option<String>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// If the upstream GraphQL server supports request batching, you can
    /// specify the 'batch' argument to batch several requests into a single
    /// batch request.
    ///
    /// Make sure you have also specified batch settings to the `@upstream` and
    /// to the `@graphQL` operator.
    pub batch: bool,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The headers parameter allows you to customize the headers of the GraphQL
    /// request made by the `@graphQL` operator. It is used by specifying a
    /// key-value map of header names and their values.
    pub headers: Vec<KeyValue>,

    /// Specifies the root field on the upstream to request data from. This maps
    /// a field in your schema to a field in the upstream schema. When a query
    /// is received for this field, Tailcall requests data from the
    /// corresponding upstream field.
    pub name: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GraphQLOperationType {
    #[default]
    Query,
    Mutation,
}

impl Display for GraphQLOperationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Query => "query",
            Self::Mutation => "mutation",
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
/// The `@const` operators allows us to embed a constant response for the
/// schema.
pub struct Const {
    pub data: Value,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
/// The @addField operator simplifies data structures and queries by adding a field that inlines or flattens a nested field or node within your schema. more info [here](https://tailcall.run/docs/guides/operators/#addfield)
pub struct AddField {
    /// Name of the new field to be added
    pub name: String,
    /// Path of the data where the field should point to
    pub path: Vec<String>,
}

impl Config {
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn from_yaml(yaml: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(yaml)?)
    }

    pub fn from_sdl(sdl: &str) -> Valid<Self, String> {
        let doc = async_graphql::parser::parse_schema(sdl);
        match doc {
            Ok(doc) => from_document(doc),
            Err(e) => Valid::fail(e.to_string()),
        }
    }

    pub fn from_source(source: Source, schema: &str) -> Result<Self> {
        match source {
            Source::GraphQL => Ok(Config::from_sdl(schema).to_result()?),
            Source::Json => Ok(Config::from_json(schema)?),
            Source::Yml => Ok(Config::from_yaml(schema)?),
        }
    }

    pub fn n_plus_one(&self) -> Vec<Vec<(String, String)>> {
        super::n_plus_one::n_plus_one(self)
    }
}

#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default, schemars::JsonSchema,
)]
pub enum Encoding {
    #[default]
    ApplicationJson,
    ApplicationXWwwFormUrlencoded,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_field_has_or_not_batch_resolver() {
        let f1 = Field { ..Default::default() };

        let f2 = Field {
            http: Some(Http { group_by: vec!["id".to_string()], ..Default::default() }),
            ..Default::default()
        };

        let f3 = Field {
            http: Some(Http { group_by: vec![], ..Default::default() }),
            ..Default::default()
        };

        assert!(!f1.has_batched_resolver());
        assert!(f2.has_batched_resolver());
        assert!(!f3.has_batched_resolver());
    }

    #[test]
    fn test_graphql_directive_name() {
        let name = GraphQL::directive_name();
        assert_eq!(name, "graphQL");
    }

    #[test]
    fn test_from_sdl_empty() {
        let actual = Config::from_sdl("type Foo {a: Int}").to_result().unwrap();
        let expected = Config::default().types(vec![(
            "Foo",
            Type::default().fields(vec![("a", Field::int())]),
        )]);
        assert_eq!(actual, expected);
    }
}
