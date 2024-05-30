# Grpc datasource with batching

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
    @grpc(
      method: "news.NewsService.GetMultipleNews"
      baseURL: "http://localhost:50051"
      body: "{{.args.news}}"
      batchKey: ["news", "id"]
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

```yml @mock
- request:
    method: POST
    url: http://localhost:50051/news.NewsService/GetMultipleNews
    textBody: \0\0\0\0\x08\n\x02\x08\x02\n\x02\x08\x03
  assertHits: false
  response:
    status: 200
    textBody: \0\0\0\0t\n#\x08\x02\x12\x06Note 2\x1a\tContent 2\"\x0cPost image 2\n#\x08\x03\x12\x06Note 3\x1a\tContent 3\"\x0cPost image 3
- request:
    method: POST
    url: http://localhost:50051/news.NewsService/GetMultipleNews
    textBody: \0\0\0\0\x08\n\x02\x08\x03\n\x02\x08\x02
  assertHits: false
  response:
    status: 200
    textBody: \0\0\0\0t\n#\x08\x03\x12\x06Note 3\x1a\tContent 3\"\x0cPost image 3\n#\x08\x02\x12\x06Note 2\x1a\tContent 2\"\x0cPost image 2
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { newsById2: newsById(news: {id: 2}) { title }, newsById3: newsById(news: {id: 3}) { title } }"
```
