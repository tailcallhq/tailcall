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

```graphql @server
schema @server(graphiql: true, port: 8000) @upstream(batch: {delay: 10, headers: [], maxSize: 100}, httpCache: true) @link(id: "news", src: "news.proto", type: Protobuf) {
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
  news: [News]!
}

type Query {
  news: NewsData! @grpc(baseURL: "http://localhost:50051", method: "news.NewsService.GetAllNews")
  newsById(news: NewsInput!): News! @grpc(baseURL: "http://localhost:50051", body: "{{args.news}}", method: "news.NewsService.GetNews")
}
```

```yml @mock
- request:
    method: POST
    url: http://localhost:50051/news.NewsService/GetAllNews
    body: null
  response:
    status: 200
    headers:
      grpc-status: 3
      grpc-message: "grpc message"
      # before base64 encoding: \x08\x03\x12\x0Derror message\x1A\x3E\x0A+type.googleapis.com/greetings.ErrValidation\x12\x0F\x0A\x0Derror details
      grpc-status-details-bin: "CAMSDWVycm9yIG1lc3NhZ2UaPgordHlwZS5nb29nbGVhcGlzLmNvbS9ncmVldGluZ3MuRXJyVmFsaWRhdGlvbhIPCg1lcnJvciBkZXRhaWxz"
    body:
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { news {news{ id }} }
```
