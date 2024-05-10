mod core;

#[cfg(feature = "cli")]
pub mod cli;

// export only what's requried outside.
pub use core::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
pub use core::blueprint::{
    Blueprint, Definition, DynamicValue, GrpcMethod, Script, Server, Type, Upstream,
};
pub use core::cache::InMemoryCache;
pub use core::config::reader::ConfigReader;
pub use core::config::{Batch, Config, ConfigModule, Source};
pub use core::endpoint::Endpoint;
pub use core::generator::Generator;
pub use core::grpc::protobuf::ProtobufSet;
pub use core::has_headers::HasHeaders;
pub use core::http::{
    handle_request, showcase, AppContext, DataLoaderRequest, HttpDataLoader, Method,
    RequestContext, RequestTemplate, Response, API_URL_PREFIX,
};
pub use core::json::{gather_path_matches, JsonLike};
pub use core::lambda::{
    EmptyResolverContext, Eval, EvaluationContext, EvaluationError, Expression, ResolverContextLike,
};
pub use core::merge_right::MergeRight;
pub use core::mustache::Mustache;
pub use core::path::PathString;
pub use core::print_schema::print_schema;
pub use core::runtime::TargetRuntime;
pub use core::scalar::{is_predefined_scalar, CUSTOM_SCALARS};
pub use core::tracing::{
    default_tracing, default_tracing_for_name, default_tracing_tailcall, get_log_level,
    tailcall_filter_target,
};
pub use core::valid::{Cause, ValidationError, Validator};
pub use core::{Cache, EntityCache, EnvIO, FileIO, HttpIO, WorkerIO};

pub use cli::runtime::NativeHttp;
