---
expect_validation_error: true
---

# test-grpc-invalid-method-format

```graphql @server
schema {
  query: Query
}

type Query {
  news: NewsData @grpc(method: "abc.NewsService", baseURL: "http://localhost:4000")
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
