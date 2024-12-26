---
error: true
---

# test-grpc-invalid-method-format

```graphql @schema
schema {
  query: Query
}

type Query {
  news: NewsData @grpc(method: "abc.NewsService", url: "http://localhost:4000")
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
