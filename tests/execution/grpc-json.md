# Grpc datasource

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
links:
  - id: "news"
    src: "news.proto"
    type: Protobuf
```

```graphql @schema
schema {
  query: Query
}

type Query {
  newsById: News! @grpc(url: "http://localhost:50051", method: "news.NewsService.GetNews", body: {id: 2})
  newsByIdMustache(news: NewsInput!): News!
    @grpc(url: "http://localhost:50051", method: "news.NewsService.GetNews", body: "{{.args.news}}")
  newsByIdMustacheAndJson(news: NewsInput!): News!
    @grpc(url: "http://localhost:50051", method: "news.NewsService.GetNews", body: {id: "{{.args.news.id}}"})
}

input NewsInput {
  id: Int
}

type News {
  id: Int
  title: String
  body: String
  postImage: String
}
```

```yml @mock
- request:
    method: POST
    url: http://localhost:50051/news.NewsService/GetNews
  response:
    status: 200
    textBody: \0\0\0\0#\x08\x02\x12\x06Note 2\x1a\tContent 2\"\x0cPost image 2
  expectedHits: 3
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { newsById { id } }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { newsByIdMustache(news: {id: 2}) { id } }"

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { newsByIdMustacheAndJson(news: {id: 2}) { id } }"
```
