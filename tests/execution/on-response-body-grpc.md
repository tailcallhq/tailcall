# onResponseBody hook on grpc directive.

```js @file:test.js
function onResponse(data) {
  const body = JSON.parse(data)
  body.title += " - Changed by JS"
  return JSON.stringify(body)
}
```

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
    rpc GetNews (NewsId) returns (News) {}
}

message NewsId {
    int32 id = 1;
}
```

```yaml @config
server:
  port: 8000
links:
  - type: Script
    src: "test.js"
  - id: "news"
    src: "news.proto"
    type: Protobuf
```

```graphql @schema
schema {
  query: Query
}

type Query {
  newsById: News!
    @grpc(
      method: "news.NewsService.GetNews"
      body: {id: 2}
      onResponseBody: "onResponse"
      url: "http://localhost:50051"
    )
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
  expectedHits: 1
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { newsById { id, title } }
```
