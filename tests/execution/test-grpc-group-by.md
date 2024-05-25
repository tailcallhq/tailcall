---
error: true
---

# test-grpc-group-by

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

```graphql @config
schema
  @server(port: 8000)
  @upstream(httpCache: 42, batch: {delay: 10})
  @link(id: "news", src: "news.proto", type: Protobuf) {
  query: Query
}

type Query {
  newsById(news: NewsInput!): News!
    @grpc(
      method: "news.NewsService.GetMultipleNews"
      baseURL: "http://localhost:50051"
      body: "{{.args.news}}"
      batchKey: ["id"]
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
```
