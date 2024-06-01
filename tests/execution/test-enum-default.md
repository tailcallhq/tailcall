# test-enum-grpc-default

```protobuf @file:service.proto
syntax = "proto3";

import "google/protobuf/empty.proto";

package news;

enum Status {
    PUBLISHED = 0;
    DRAFT = 1;
    NOT_DEFINED = 2;
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

```graphql @config
# for test upstream server see [repo](https://github.com/tailcallhq/rust-grpc)
schema
  @server(port: 8080)
  @upstream(baseURL: "http://localhost:50051", httpCache: 42, batch: {delay: 10})
  @link(id: "news", src: "./service.proto", type: Protobuf) {
  query: Query
}

type Query {
  news: NewsData! @grpc(method: "news.NewsService.GetAllNews")
}

enum Status {
  PUBLISHED
  DRAFT
  NOT_DEFINED
}

type News {
  id: Int
  foo: Status
}

input NewsInput {
  id: Int
}

type NewsData {
  news: [News]!
}
```

```yml @mock
- request:
    method: POST
    url: http://localhost:50051/news.NewsService/GetAllNews
    textBody: \0\0\0\0\0
  response:
    status: 200
    textBody: '\0\0\0\0s\n#\x08\x01\x12\x06Note 1\x1a\tContent 1\"\x0cPost image 1\n%\x08\x02\x12\x06Note 2\x1a\tContent 2\"\x0cPost image 2(\x01\n%\x08\x03\x12\x06Note 3\x1a\tContent 3\"\x0cPost image 3(\x02'
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { news { news { id foo } } }"
```
