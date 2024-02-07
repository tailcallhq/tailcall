# test-missing-argument-on-all-resolvers

###### sdl error

#### file:news.proto

```protobuf
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

#### server:

```graphql
schema @upstream(baseURL: "http://jsonplaceholder.typicode.com") @link(id: "news", src: "news.proto", type: Protobuf) {
  query: Query
}

type Post {
  id: Int!
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
  postGraphQLArgs: Post @graphQL(name: "post", args: [{key: "id", value: "{{args.id}}"}])
  postGraphQLHeaders: Post @graphQL(name: "post", headers: [{key: "id", value: "{{args.id}}"}])
  postHttp: Post @http(path: "/posts/{{args.id}}")
  newsGrpcHeaders: NewsData! @grpc(method: "news.NewsService.GetAllNews", headers: [{key: "id", value: "{{args.id}}"}])
  newsGrpcUrl: NewsData! @grpc(method: "news.NewsService.GetAllNews", baseURL: "{{args.url}}")
  newsGrpcBody: NewsData! @grpc(method: "news.NewsService.GetAllNews", body: "{{args.id}}")
}

type User {
  id: Int
  name: String
}
```
