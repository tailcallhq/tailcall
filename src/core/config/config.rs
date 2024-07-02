use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::{self, Display};
use std::num::NonZeroU64;

use anyhow::Result;
use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::telemetry::Telemetry;
use super::{KeyValue, Link, Server, Upstream};
use crate::core::config::from_document::from_document;
use crate::core::config::source::Source;
use crate::core::directive::DirectiveCodec;
use crate::core::http::Method;
use crate::core::json::JsonSchema;
use crate::core::macros::MergeRight;
use crate::core::merge_right::MergeRight;
use crate::core::valid::{Valid, Validator};
use crate::core::{is_default, scalar};

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    Setters,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    MergeRight,
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
    /// A map of all the enum types in the schema
    #[serde(default, skip_serializing_if = "is_default")]
    pub enums: BTreeMap<String, Enum>,

    ///
    /// A list of all links in the schema.
    #[serde(default, skip_serializing_if = "is_default")]
    pub links: Vec<Link>,
    #[serde(default, skip_serializing_if = "is_default")]
    /// Enable [opentelemetry](https://opentelemetry.io) support
    pub telemetry: Telemetry,
}

///
/// Represents a GraphQL type.
/// A type can be an object, interface, enum or scalar.
#[derive(
    Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema, MergeRight,
)]
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
    /// Interfaces that the type implements.
    pub implements: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    ///
    /// Setting to indicate if the type can be cached.
    pub cache: Option<Cache>,
    ///
    /// Marks field as protected by auth providers
    #[serde(default)]
    pub protected: Option<Protected>,
    #[serde(default, skip_serializing_if = "is_default")]
    ///
    /// Contains source information for the type.
    pub tag: Option<Tag>,
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

    pub fn scalar(&self) -> bool {
        self.fields.is_empty()
    }
}

#[derive(
    Clone, Debug, Default, PartialEq, Deserialize, Serialize, Eq, schemars::JsonSchema, MergeRight,
)]
#[serde(deny_unknown_fields)]
/// Used to represent an identifier for a type. Typically used via only by the
/// configuration generators to provide additional information about the type.
pub struct Tag {
    /// A unique identifier for the type.
    pub id: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Eq, schemars::JsonSchema, MergeRight)]
/// The @cache operator enables caching for the query, field or type it is
/// applied to.
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Cache {
    /// Specifies the duration, in milliseconds, of how long the value has to be
    /// stored in the cache.
    pub max_age: NonZeroU64,
}

#[derive(
    Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default, schemars::JsonSchema, MergeRight,
)]
pub struct Protected {}

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    Setters,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    MergeRight,
)]
#[setters(strip_option)]
pub struct RootSchema {
    pub query: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub mutation: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub subscription: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
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
    #[serde(rename = "expr", default, skip_serializing_if = "is_default")]
    pub const_field: Option<Expr>,

    ///
    /// Inserts a GraphQL resolver for the field.
    #[serde(default, skip_serializing_if = "is_default")]
    pub graphql: Option<GraphQL>,

    ///
    /// Sets the cache configuration for a field
    pub cache: Option<Cache>,

    ///
    /// Marks field as protected by auth provider
    #[serde(default)]
    pub protected: Option<Protected>,

    ///
    /// Stores the default value for the field
    #[serde(default, skip_serializing_if = "is_default")]
    pub default_value: Option<Value>,
}

// It's a terminal implementation of MergeRight
impl MergeRight for Field {
    fn merge_right(self, other: Self) -> Self {
        other
    }
}

impl Field {
    pub fn has_resolver(&self) -> bool {
        self.http.is_some()
            || self.script.is_some()
            || self.const_field.is_some()
            || self.graphql.is_some()
            || self.grpc.is_some()
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
            directives.push(Expr::trace_name());
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
    pub fn into_list(mut self) -> Self {
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
        self.omit.is_some()
            || self
                .modify
                .as_ref()
                .and_then(|m| m.omit)
                .unwrap_or_default()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
pub struct JS {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
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

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
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

#[derive(
    Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema, MergeRight,
)]
pub struct Union {
    pub types: BTreeSet<String>,
    pub doc: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema, MergeRight)]
/// Definition of GraphQL enum type
pub struct Enum {
    pub variants: BTreeSet<Variant>,
    pub doc: Option<String>,
}

/// Definition of GraphQL value
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    schemars::JsonSchema,
    MergeRight,
)]
pub struct Variant {
    pub name: String,
    // directive: alias
    pub alias: Option<Alias>,
}

/// The @alias directive indicates that aliases of one enum value.
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    schemars::JsonSchema,
    MergeRight,
)]
pub struct Alias {
    pub options: BTreeSet<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// The @http operator indicates that a field or node is backed by a REST API.
///
/// For instance, if you add the @http operator to the `users` field of the
/// Query type with a path argument of `"/users"`, it signifies that the `users`
/// field is backed by a REST API. The path argument specifies the path of the
/// REST API. In this scenario, the GraphQL server will make a GET request to
/// the API endpoint specified when the `users` field is queried.
pub struct Http {
    #[serde(rename = "onRequest", default, skip_serializing_if = "is_default")]
    /// onRequest field in @http directive gives the ability to specify the
    /// request interception handler.
    pub on_request: Option<String>,

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
/// Provides the ability to refer to multiple fields in the Query or
/// Mutation root.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
pub struct Call {
    /// Steps are composed together to form a call.
    /// If you have multiple steps, the output of the previous step is passed as
    /// input to the next step.
    pub steps: Vec<Step>,
}

///
/// Provides the ability to refer to a field defined in the root Query or
/// Mutation.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
pub struct Step {
    #[serde(default, skip_serializing_if = "is_default")]
    /// The name of the field on the `Query` type that you want to call.
    pub query: Option<String>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The name of the field on the `Mutation` type that you want to call.
    pub mutation: Option<String>,

    /// The arguments that will override the actual arguments of the field.
    #[serde(default, skip_serializing_if = "is_default")]
    pub args: BTreeMap<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
/// The `@expr` operators allows you to specify an expression that can evaluate
/// to a value. The expression can be a static value or built form a Mustache
/// template. schema.
pub struct Expr {
    pub body: Value,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// The @addField operator simplifies data structures and queries by adding a field that inlines or flattens a nested field or node within your schema. more info [here](https://tailcall.run/docs/guides/operators/#addfield)
pub struct AddField {
    /// Name of the new field to be added
    pub name: String,
    /// Path of the data where the field should point to
    pub path: Vec<String>,
}

impl Config {
    pub fn is_root_operation_type(&self, type_name: &str) -> bool {
        let type_name = type_name.to_lowercase();

        [
            &self.schema.query,
            &self.schema.mutation,
            &self.schema.subscription,
        ]
        .iter()
        .filter_map(|&root_name| root_name.as_ref())
        .any(|root_name| root_name.to_lowercase() == type_name)
    }

    pub fn port(&self) -> u16 {
        self.server.port.unwrap_or(8000)
    }

    pub fn find_type(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }

    pub fn find_union(&self, name: &str) -> Option<&Union> {
        self.unions.get(name)
    }

    pub fn find_enum(&self, name: &str) -> Option<&Enum> {
        self.enums.get(name)
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

    /// Renders current config to graphQL string
    pub fn to_sdl(&self) -> String {
        crate::core::document::print(self.into())
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
        self.types.contains_key(name)
            || self.unions.contains_key(name)
            || self.enums.contains_key(name)
    }

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

    ///
    /// Given a starting type, this function searches for all the unique types
    /// that this type can be connected to via it's fields
    fn find_connections(&self, type_of: &str, mut types: HashSet<String>) -> HashSet<String> {
        if let Some(union_) = self.find_union(type_of) {
            types.insert(type_of.into());

            for type_ in union_.types.iter() {
                types = self.find_connections(type_, types);
            }
        } else if let Some(type_) = self.find_type(type_of) {
            types.insert(type_of.into());
            for (_, field) in type_.fields.iter() {
                if !types.contains(&field.type_of) && !self.is_scalar(&field.type_of) {
                    types = self.find_connections(&field.type_of, types);
                }
            }
        }
        types
    }

    ///
    /// Checks if a type is a scalar or not.
    pub fn is_scalar(&self, type_name: &str) -> bool {
        self.types
            .get(type_name)
            .map_or(scalar::is_predefined_scalar(type_name), |ty| ty.scalar())
    }

    ///
    /// Goes through the complete config and finds all the types that are used
    /// as inputs directly ot indirectly.
    pub fn input_types(&self) -> HashSet<String> {
        self.arguments()
            .iter()
            .filter(|(_, arg)| !self.is_scalar(&arg.type_of))
            .map(|(_, arg)| arg.type_of.as_str())
            .fold(HashSet::new(), |types, type_of| {
                self.find_connections(type_of, types)
            })
    }

    /// finds the all types which are present in union.
    pub fn union_types(&self) -> HashSet<String> {
        self.unions
            .values()
            .flat_map(|union| union.types.iter().cloned())
            .collect()
    }

    /// Returns a list of all the types that are used as output types
    pub fn output_types(&self) -> HashSet<String> {
        let mut types = HashSet::new();

        if let Some(ref query) = &self.schema.query {
            types = self.find_connections(query, types);
        }

        if let Some(ref mutation) = &self.schema.mutation {
            types = self.find_connections(mutation, types);
        }

        types
    }

    /// Returns a list of all the types that are used as interface
    pub fn interface_types(&self) -> HashSet<String> {
        let mut types = HashSet::new();

        for ty in self.types.values() {
            for interface in ty.implements.iter() {
                types.insert(interface.clone());
            }
        }

        types
    }

    /// Returns a list of all the arguments in the configuration
    fn arguments(&self) -> Vec<(&String, &Arg)> {
        self.types
            .iter()
            .flat_map(|(_, type_of)| type_of.fields.iter())
            .flat_map(|(_, field)| field.args.iter())
            .collect::<Vec<_>>()
    }
    /// Removes all types that are passed in the set
    pub fn remove_types(mut self, types: HashSet<String>) -> Self {
        for unused_type in types {
            self.types.remove(&unused_type);
            self.unions.remove(&unused_type);
        }

        self
    }

    pub fn unused_types(&self) -> HashSet<String> {
        let used_types = self.get_all_used_type_names();
        let all_types: HashSet<String> = self
            .types
            .keys()
            .chain(self.unions.keys())
            .cloned()
            .collect();
        all_types.difference(&used_types).cloned().collect()
    }

    /// Gets all the type names used in the schema.
    pub fn get_all_used_type_names(&self) -> HashSet<String> {
        let mut set = HashSet::new();
        let mut stack = Vec::new();
        if let Some(query) = &self.schema.query {
            stack.push(query.clone());
        }
        if let Some(mutation) = &self.schema.mutation {
            stack.push(mutation.clone());
        }
        while let Some(type_name) = stack.pop() {
            if set.contains(&type_name) {
                continue;
            }
            if let Some(union_) = self.unions.get(&type_name) {
                set.insert(type_name);
                for type_ in &union_.types {
                    stack.push(type_.clone());
                }
            } else if let Some(typ) = self.types.get(&type_name) {
                set.insert(type_name);
                for field in typ.fields.values() {
                    stack.extend(field.args.values().map(|arg| arg.type_of.clone()));
                    stack.push(field.type_of.clone());
                }
            }
        }

        set
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

    #[test]
    fn test_unused_types_with_cyclic_types() {
        let config = Config::from_sdl(
            "
            type Bar {a: Int}
            type Foo {a: [Foo]}

            type Query {
                foos: [Foo]
            }

            schema {
                query: Query
            }
            ",
        )
        .to_result()
        .unwrap();

        let actual = config.unused_types();
        let mut expected = HashSet::new();
        expected.insert("Bar".to_string());

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_is_root_operation_type_with_query() {
        let mut config = Config::default();
        config.schema.query = Some("Query".to_string());

        assert!(config.is_root_operation_type("Query"));
        assert!(!config.is_root_operation_type("Mutation"));
        assert!(!config.is_root_operation_type("Subscription"));
    }

    #[test]
    fn test_is_root_operation_type_with_mutation() {
        let mut config = Config::default();
        config.schema.mutation = Some("Mutation".to_string());

        assert!(!config.is_root_operation_type("Query"));
        assert!(config.is_root_operation_type("Mutation"));
        assert!(!config.is_root_operation_type("Subscription"));
    }

    #[test]
    fn test_is_root_operation_type_with_subscription() {
        let mut config = Config::default();
        config.schema.subscription = Some("Subscription".to_string());

        assert!(!config.is_root_operation_type("Query"));
        assert!(!config.is_root_operation_type("Mutation"));
        assert!(config.is_root_operation_type("Subscription"));
    }

    #[test]
    fn test_is_root_operation_type_with_no_root_operation() {
        let config = Config::default();

        assert!(!config.is_root_operation_type("Query"));
        assert!(!config.is_root_operation_type("Mutation"));
        assert!(!config.is_root_operation_type("Subscription"));
    }

    #[test]
    fn test_union_types() {
        let sdl = std::fs::read_to_string(tailcall_fixtures::configs::UNION_CONFIG).unwrap();
        let config = Config::from_sdl(&sdl).to_result().unwrap();
        let union_types = config.union_types();
        let expected_union_types: HashSet<String> = ["Bar", "Baz", "Foo"]
            .iter()
            .cloned()
            .map(String::from)
            .collect();
        assert_eq!(union_types, expected_union_types);
    }
}
