---
expect_validation_error: true
---

# test-missing-argument-on-all-resolvers

```protobuf @file:news.proto
syntax = "proto3";

import "google/protobuf/empty.proto";

package news;

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

```graphql @server
schema @upstream(baseURL: "http://jsonplaceholder.typicode.com") @link(id: "news", src: "news.proto", type: Protobuf) {
  query: Query
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

type Post {
  id: Int!
}

type Query {
  newsGrpcBody: NewsData! @grpc(body: "{{args.id}}", method: "news.NewsService.GetAllNews")
  newsGrpcHeaders: NewsData! @grpc(headers: [{key: "id", value: "{{args.id}}"}], method: "news.NewsService.GetAllNews")
  newsGrpcUrl: NewsData! @grpc(baseURL: "{{args.url}}", method: "news.NewsService.GetAllNews")
  postGraphQLArgs: Post @graphQL(args: [{key: "id", value: "{{args.id}}"}], name: "post")
  postGraphQLHeaders: Post @graphQL(headers: [{key: "id", value: "{{args.id}}"}], name: "post")
  postHttp: Post @http(path: "/posts/{{args.id}}")
}

type User {
  id: Int
  name: String
}
```
