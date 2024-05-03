---
expect_validation_error: true
---

# test-grpc-invalid-proto-id

```graphql @server
schema {
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
  news: NewsData @grpc(baseURL: "http://localhost:4000", method: "abc.NewsService.GetAllNews")
}
```
