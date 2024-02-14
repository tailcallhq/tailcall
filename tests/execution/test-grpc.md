# test-grpc

###### check identity

#### file:news.proto

```protobuf
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
```

#### file:package.proto

```
syntax = "proto3";

package com.protos.greetings;

// The greeter service definition.
service Greeter {
  // Sends a greeting
  rpc SayHello (HelloRequest) returns (HelloReply) {}
}

// The request message containing the user's name.
message HelloRequest {
  string name = 1;
}

// The response message containing the greetings
message HelloReply {
  string message = 1;
}
```

#### server:

```graphql
schema @server(port: 8000) @upstream(baseURL: "http://localhost:50051", batch: {delay: 10, headers: [], maxSize: 1000}) @link(id: "news", src: "news.proto", type: Protobuf) @link(id: "package", src: "package.proto", type: Protobuf) {
  query: Query
}

input HelloRequest {
  name: String!
}

input NewsInput {
  body: String
  id: Int
  postImage: String
  title: String
}

type HelloReply {
  message: String
}

type News {
  body: String
  id: Int
  postImage: String
  title: String
}

type NewsData {
  news: [News]!
}

type Query {
  greetings(request: HelloRequest): HelloReply! @grpc(method: "package.com.protos.greetings.Greeter.SayHello")
  news: NewsData! @grpc(method: "news.NewsService.GetAllNews")
  newsById(news: NewsInput!): News! @grpc(body: "{{args.news}}", method: "news.NewsService.GetNews")
  newsByIdBatch(news: NewsInput!): News! @grpc(body: "{{args.news}}", groupBy: ["news", "id"], method: "news.NewsService.GetMultipleNews")
}
```
