# test-grpc-invalid-proto-id

---

expect_validation_error: true

---

```graphql @server
schema {
  query: Query
}

type Query {
  news: NewsData @grpc(method: "abc.NewsService.GetAllNews", baseURL: "http://localhost:4000")
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
