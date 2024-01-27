use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use anyhow::anyhow;
use lazy_static::lazy_static;
use tailcall::async_graphql_hyper::GraphQLRequest;
use tailcall::blueprint::Blueprint;
use tailcall::config::reader::ConfigReader;
use tailcall::config::ConfigSet;
use tailcall::http::{graphiql, handle_request, AppContext};
use tailcall::EnvIO;

use crate::http::{to_request, to_response};
use crate::{init_cache, init_env, init_file, init_http, init_proto_resolver};

lazy_static! {
    static ref APP_CTX: RwLock<Option<(String, Arc<AppContext>)>> = RwLock::new(None);
}
///
/// The handler which handles requests on cloudflare
///
pub async fn fetch(req: worker::Request, env: worker::Env) -> anyhow::Result<worker::Response> {
    log::info!(
        "{} {:?}",
        req.method().to_string(),
        req.url().map(|u| u.to_string())
    );
    let env = Rc::new(env);

    let file_io = init_file(env.clone(), String::from("MY_R2"))?;
    let grpc_gql = r#"
    # for test upstream server see [repo](https://github.com/tailcallhq/node-grpc)
schema @server(port: 8000, graphiql: true) @upstream(httpCache: true, batch: {delay: 10}) {
  query: Query
}

type Query {
  news: NewsData!
    @grpc(
      service: "NewsService"
      method: "GetAllNews"
      baseURL: "http://0.0.0.0:50051"
      protoPath: "src/grpc/tests/news.proto"
    )
  newsById(news: NewsInput!): News!
    @grpc(
      service: "NewsService"
      method: "GetNews"
      baseURL: "http://0.0.0.0:50051"
      body: "{{args.news}}"
      protoPath: "src/grpc/tests/news.proto"
    )
  newsByIdBatch(news: NewsInput!): News!
    @grpc(
      service: "NewsService"
      method: "GetMultipleNews"
      baseURL: "http://0.0.0.0:50051"
      body: "{{args.news}}"
      protoPath: "src/grpc/tests/news.proto"
      groupBy: ["news", "id"]
    )
}
input NewsInput {
  id: Int
  title: String
  body: String
  postImage: String
}
type NewsData {
  news: [News]!
}

type News {
  id: Int
  title: String
  body: String
  postImage: String
}
    "#;
    file_io.write("examples/grpc.graphql", grpc_gql.as_bytes()).await?;
    let news = r#"
    syntax = "proto3";

import "google/protobuf/empty.proto";

message News {
    int32 id = 1;
    string title = 2;
    string body = 3;
    string postImage = 4;
}

service NewsService {
    rpc GetAllNews (google.protobuf.Empty) returns (NewsList) {}
    rpc GetNews (NewsId) returns (News) {}
    rpc GetMultipleNews (MultipleNewsId) returns (NewsList) {}
    rpc DeleteNews (NewsId) returns (google.protobuf.Empty) {}
    rpc EditNews (News) returns (News) {}
    rpc AddNews (News) returns (News) {}
}

message NewsId {
    int32 id = 1;
}

message MultipleNewsId {
    repeated NewsId ids = 1;
}

message NewsList {
   repeated News news = 1;
}
    "#;
    file_io.write("src/grpc/tests/news.proto", news.as_bytes()).await?;
    let empty = r#"
    // Protocol Buffers - Google's data interchange format
// Copyright 2008 Google Inc.  All rights reserved.
// https://developers.google.com/protocol-buffers/
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//     * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//     * Neither the name of Google Inc. nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

syntax = "proto3";

package google.protobuf;

option csharp_namespace = "Google.Protobuf.WellKnownTypes";
option go_package = "github.com/golang/protobuf/ptypes/empty";
option java_package = "com.google.protobuf";
option java_outer_classname = "EmptyProto";
option java_multiple_files = true;
option objc_class_prefix = "GPB";
option cc_enable_arenas = true;

// A generic empty message that you can re-use to avoid defining duplicated
// empty messages in your APIs. A typical example is to use it as the request
// or the response type of an API method. For instance:
//
//     service Foo {
//       rpc Bar(google.protobuf.Empty) returns (google.protobuf.Empty);
//     }
//
// The JSON representation for `Empty` is empty JSON object `{}`.
message Empty {}

    "#;
    file_io.write("google/protobuf/empty.proto", empty.as_bytes()).await?;

    let hyper_req = to_request(req).await?;
    if hyper_req.method() == hyper::Method::GET {
        let response = graphiql(&hyper_req)?;
        return to_response(response).await;
    }
    let query = hyper_req
        .uri()
        .query()
        .ok_or(anyhow!("Unable parse extract query"))?;
    let query = serde_qs::from_str::<HashMap<String, String>>(query)?;
    let config_path = query
        .get("config")
        .ok_or(anyhow!("The key 'config' not found in the query"))?
        .clone();

    log::info!("config-url: {}", config_path);
    let app_ctx = get_app_ctx(env, config_path.as_str()).await?;
    let resp = handle_request::<GraphQLRequest>(hyper_req, app_ctx).await?;

    Ok(to_response(resp).await?)
}

///
/// Reads the configuration from the CONFIG environment variable.
///
async fn get_config(
    env_io: Arc<dyn EnvIO>,
    env: Rc<worker::Env>,
    file_path: &str,
) -> anyhow::Result<ConfigSet> {
    let bucket_id = env_io
        .get("BUCKET")
        .ok_or(anyhow!("BUCKET var is not set"))?;
    log::debug!("R2 Bucket ID: {}", bucket_id);
    let file_io = init_file(env.clone(), bucket_id)?;
    let http_io = init_http();
    let proto_resolver = init_proto_resolver();

    let reader = ConfigReader::init(file_io, http_io, proto_resolver);
    let config = reader.read(&file_path).await?;
    Ok(config)
}

///
/// Initializes the worker once and caches the app context
/// for future requests.
///
async fn get_app_ctx(env: Rc<worker::Env>, file_path: &str) -> anyhow::Result<Arc<AppContext>> {
    // Read context from cache
    if let Some(app_ctx) = read_app_ctx() {
        if app_ctx.0 == file_path {
            log::info!("Using cached application context");
            return Ok(app_ctx.clone().1);
        }
    }
    // Create new context
    let env_io = init_env(env.clone());
    let cfg = get_config(env_io.clone(), env.clone(), file_path).await?;
    log::info!("Configuration read ... ok");
    log::debug!("\n{}", cfg.to_sdl());
    let blueprint = Blueprint::try_from(&cfg)?;
    log::info!("Blueprint generated ... ok");
    let h_client = init_http();
    let cache = init_cache(env);
    let app_ctx = Arc::new(AppContext::new(
        blueprint,
        h_client.clone(),
        h_client,
        env_io,
        cache,
        None,
    ));
    *APP_CTX.write().unwrap() = Some((file_path.to_string(), app_ctx.clone()));
    log::info!("Initialized new application context");
    Ok(app_ctx)
}

fn read_app_ctx() -> Option<(String, Arc<AppContext>)> {
    APP_CTX.read().unwrap().clone()
}
