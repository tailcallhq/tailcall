---
error: true
---

# test-grpc-proto-path

```graphql @server
schema @link(id: "news", src: "tailcall/src/grpcnews.proto", type: Protobuf) {
  query: Query
}

type Query {
  news: NewsData @grpc(method: "GetAllNews", baseURL: "http://localhost:4000")
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
