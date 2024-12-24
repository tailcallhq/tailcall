---
identity: true
---

# test-grpc

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

```yaml @config
server:
  port: 8000
upstream:
  batch:
    delay: 10
    maxSize: 1000
links:
  - id: "news"
    src: "news.proto"
    type: Protobuf
```

```graphql @schema
schema @server @upstream {
  query: Query
}

input NewsInput {
  body: String
  id: Int
  postImage: String
  title: String
}

type News {
  body: String
  id: Int
  postImage: String
  title: String
}

type NewsData {
  news: [News]
}

type Query {
  news: NewsData! @grpc(url: "http://localhost:50051", method: "news.NewsService.GetAllNews")
  newsById(news: NewsInput!): News!
    @grpc(url: "http://localhost:50051", body: "{{.args.news}}", method: "news.NewsService.GetNews")
  newsByIdBatch(news: NewsInput!): News!
    @grpc(
      url: "http://localhost:50051"
      body: "{{.args.news}}"
      batchKey: ["news", "id"]
      method: "news.NewsService.GetMultipleNews"
    )
}
```
