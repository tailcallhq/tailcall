---
error: true
---

# test-grpc-invalid-proto-id

```graphql @schema
schema {
  query: Query
}

type Query {
  news: NewsData @grpc(method: "abc.NewsService.GetAllNews", url: "http://localhost:4000")
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
