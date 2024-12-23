---
error: true
---

# test-grpc-proto-path

```graphql @schema
schema @link(id: "news", src: "tailcall/src/grpcnews.proto", type: Protobuf) {
  query: Query
}

type Query {
  news: NewsData @grpc(method: "GetAllNews", url: "http://localhost:4000")
}

type NewsData {
  news: [News]
}

type News {
  id: Int!
  title: String!
  body: String!
  postImage: String!
}
```
