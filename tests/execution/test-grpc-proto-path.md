---
expect_validation_error: true
---

# test-grpc-proto-path

```graphql @server
schema @link(id: "news", src: "tailcall/src/grpcnews.proto", type: Protobuf) {
  query: Query
}

type News {
  body: String!
  id: Int!
  postImage: String!
  title: String!
}

type NewsData {
  news: [News]
}

type Query {
  news: NewsData @grpc(baseURL: "http://localhost:4000", method: "GetAllNews")
}
```
