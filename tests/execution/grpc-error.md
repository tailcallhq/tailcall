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

```graphql @config
schema
  @server(port: 8000)
  @upstream(httpCache: 42, batch: {delay: 10})
  @link(id: "news", src: "news.proto", type: Protobuf) {
  query: Query
}

type Query {
  news: NewsData! @grpc(method: "news.NewsService.GetAllNews", baseURL: "http://localhost:50051")
  newsById(news: NewsInput!): News!
    @grpc(method: "news.NewsService.GetNews", baseURL: "http://localhost:50051", body: "{{.args.news}}")
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

```yml @mock
- request:
    method: POST
    url: http://localhost:50051/news.NewsService/GetAllNews
  response:
    status: 200
    headers:
      grpc-status: 3
      grpc-message: "grpc message"
      # before base64 encoding: \x08\x03\x12\x0Derror message\x1A\x3E\x0A+type.googleapis.com/greetings.ErrValidation\x12\x0F\x0A\x0Derror details
      grpc-status-details-bin: "CAMSDWVycm9yIG1lc3NhZ2UaPgordHlwZS5nb29nbGVhcGlzLmNvbS9ncmVldGluZ3MuRXJyVmFsaWRhdGlvbhIPCg1lcnJvciBkZXRhaWxz"
    body:
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { news {news{ id }} }
```
