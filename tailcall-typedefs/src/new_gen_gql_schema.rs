use async_graphql_parser::types::ServiceDocument;
use schemars::schema::{InstanceType, RootSchema, Schema, SchemaObject, SingleOrVec};
use tailcall::core::config::cors::Cors;
use tailcall::core::config::headers::Headers;
use tailcall::core::config::{
    AddField, Alias, Apollo, Batch, Cache, Call, Config, Expr, GraphQL, Grpc, Http, KeyValue, Link,
    Modify, Omit, OtlpExporter, PrometheusExporter, PrometheusFormat, Protected, Proxy,
    ScriptOptions, Server, StdoutExporter, Step, Tag, Telemetry, TelemetryExporter, Upstream, JS,
};
use tailcall::core::json::JsonSchema;
use tailcall::core::scalar::{
    Bytes, Date, Email, Empty, Int128, Int16, Int32, Int64, Int8, PhoneNumber, UInt128, UInt16,
    UInt32, UInt64, UInt8, Url, JSON,
};
use tailcall_typedefs_common::DocumentDefinition;

fn to_service_doc() -> ServiceDocument {
    let mut doc = ServiceDocument { definitions: vec![] };

    macro_rules! to_service_doc {
        ($ty:ty) => {
            doc = <$ty>::definition(doc)
        };
    }

    // directives
    to_service_doc!(AddField);
    to_service_doc!(Alias);
    to_service_doc!(Cache);
    to_service_doc!(Call);
    to_service_doc!(Expr);
    to_service_doc!(GraphQL);
    to_service_doc!(Grpc);
    to_service_doc!(Http);
    to_service_doc!(JS);
    to_service_doc!(Link);
    to_service_doc!(Modify);
    to_service_doc!(Omit);
    to_service_doc!(Protected);
    to_service_doc!(Server);
    to_service_doc!(Tag);
    to_service_doc!(Telemetry);
    to_service_doc!(Upstream);

    // default scalars
    to_service_doc!(Bytes);
    to_service_doc!(Email);
    to_service_doc!(Date);
    to_service_doc!(PhoneNumber);
    to_service_doc!(Url);
    to_service_doc!(JSON);
    to_service_doc!(Empty);
    to_service_doc!(Int8);
    to_service_doc!(Int16);
    to_service_doc!(Int32);
    to_service_doc!(Int64);
    to_service_doc!(Int128);
    to_service_doc!(UInt8);
    to_service_doc!(UInt16);
    to_service_doc!(UInt32);
    to_service_doc!(UInt64);
    to_service_doc!(UInt128);

    // inputs
    to_service_doc!(Batch);
    to_service_doc!(Apollo);
    to_service_doc!(Cors);
    to_service_doc!(Headers);
    to_service_doc!(KeyValue);
    to_service_doc!(OtlpExporter);
    to_service_doc!(PrometheusExporter);
    to_service_doc!(PrometheusFormat);
    to_service_doc!(Proxy);
    to_service_doc!(ScriptOptions);
    to_service_doc!(StdoutExporter);
    to_service_doc!(Step);
    to_service_doc!(TelemetryExporter);

    //to_service_doc!(JsonSchema);

    doc
}
