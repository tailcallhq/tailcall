# test-enum-grpc-default

```protobuf @file:service.proto
syntax = "proto3";

import "google/protobuf/empty.proto";

package news;

enum Status {
  PUBLISHED = 0;
  DRAFT = 1;
  ND = 2;
}


message News {
  int32 id = 1;
  Status foo = 5;
}

service NewsService {
  rpc GetAllNews (google.protobuf.Empty) returns (NewsList) {}
}

message NewsList {
  repeated News news = 1;
}
```

```graphql @server
schema @server(graphiql: true, port: 8080) @upstream(baseURL: "http://localhost:50051", batch: {delay: 10, headers: [], maxSize: 100}, httpCache: true) @link(id: "news", src: "./service.proto", type: Protobuf) {
  query: Query
}

enum Status {
  DRAFT
  ND
  PUBLISHED
}

type News {
  foo: Status
  id: Int
}

type NewsData {
  news: [News]!
}

type NewsInput {
  id: Int
}

type Query {
  news: NewsData! @grpc(method: "news.NewsService.GetAllNews")
}
```

```yml @mock
- request:
    method: POST
    url: http://localhost:50051/news.NewsService/GetAllNews
    body: '\0\0\0\0\0'
  response:
    status: 200
    body: '\0\0\0\0s\n#\x08\x01\x12\x06Note 1\x1a\tContent 1\"\x0cPost image 1\n%\x08\x02\x12\x06Note 2\x1a\tContent 2\"\x0cPost image 2(\x01\n%\x08\x03\x12\x06Note 3\x1a\tContent 3\"\x0cPost image 3(\x02'
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { news { news { id foo } } }"
```
