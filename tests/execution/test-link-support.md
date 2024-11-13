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

```yml @config
schema: {}
upstream:
  batch: {delay: 10, headers: [], maxSize: 1000}
links:
  - id: "news"
    src: "news.proto"
    type: Protobuf
```

```graphql @schema
schema {
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
  newsById(news: NewsInput!): News!
    @grpc(url: "http://localhost:50051", body: "{{.args.news}}", method: "news.NewsService.GetNews")
}
```
