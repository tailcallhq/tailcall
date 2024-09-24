use lazy_static::lazy_static;

mod add_field;
mod alias;
mod cache;
mod call;
mod expr;
mod federation;
mod graphql;
mod grpc;
mod http;
mod js;
mod link;
mod modify;
mod omit;
mod protected;
mod server;
mod telemetry;
mod upstream;

pub use add_field::*;
pub use alias::*;
pub use cache::*;
pub use call::*;
pub use expr::*;
pub use federation::*;
pub use graphql::*;
pub use grpc::*;
pub use http::*;
pub use js::*;
pub use link::*;
pub use modify::*;
pub use omit::*;
pub use protected::*;
pub use server::*;
pub use telemetry::*;
pub use upstream::*;

use crate::core::directive::DirectiveCodec;

lazy_static! {
    pub static ref KNOWN_DIRECTIVES: Vec<String> = vec![
        add_field::AddField::directive_name(),
        alias::Alias::directive_name(),
        cache::Cache::directive_name(),
        call::Call::directive_name(),
        expr::Expr::directive_name(),
        graphql::GraphQL::directive_name(),
        grpc::Grpc::directive_name(),
        http::Http::directive_name(),
        js::JS::directive_name(),
        link::Link::directive_name(),
        modify::Modify::directive_name(),
        omit::Omit::directive_name(),
        protected::Protected::directive_name(),
        server::Server::directive_name(),
        telemetry::Telemetry::directive_name(),
        upstream::Upstream::directive_name(),
    ];
}
