---
identity: true
---

# test-link-support

```protobuf @file:news.proto
syntax = "proto3";

import "google/protobuf/empty.proto";

package news;

message News {
    int32 id = 1;
}

service NewsService {
    rpc GetNews (NewsId) returns (News) {}
}

message NewsId {
    int32 id = 1;
}
```

```graphql @config
schema
  @server(port: 8000)
  @upstream(baseURL: "http://localhost:50051", batch: {delay: 10, headers: [], maxSize: 1000})
  @link(id: "news", src: "news.proto", meta: {description: "Test"}, type: Protobuf) {
  query: Query
}

input NewsInput {
  id: Int
}

type News {
  id: Int
}

type NewsData {
  news: [News]
}

type Query {
  newsById(news: NewsInput!): News! @grpc(body: "{{.args.news}}", method: "news.NewsService.GetNews")
}
```
