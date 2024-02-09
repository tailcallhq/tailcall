# test-expr-success

###### check identity

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
schema @server(port: 8000) @upstream(baseURL: "http://localhost:50051", batch: {delay: 10, headers: [], maxSize: 1000}) @link(id: "news", src: "news.proto", type: Protobuf) {
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

type Post {
  content: String @expr(body: {graphQL: {args: [{key: "id", value: "{{value.id}}"}], name: "postContent"}})
  id: Int!
}

type Query {
  cond: Post @expr(body: {if: {cond: {const: true}, else: {http: {path: "/posts/1"}}, then: {http: {path: "/posts/2"}}}})
  greeting: String @expr(body: {const: "hello from server"})
  news(news: NewsInput!): News! @expr(body: {grpc: {body: "{{args.news}}", groupBy: ["news", "id"], method: "news.NewsService.GetMultipleNews"}})
  post(id: Int!): Post @expr(body: {http: {baseURL: "http://jsonplacheholder.typicode.com", path: "/posts/{{args.id}}"}})
}
```
