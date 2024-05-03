---
expect_validation_error: true
---

# test-lack-resolver

```graphql @server
schema @server(port: 8000) @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type InPost {
  get: [Post]
}

type Post {
  body: String!
  id: Int!
  title: String!
  user: User @http(path: "/users/1")
  userId: Int!
}

type Query {
  posts: InPost
}

type User {
  id: Int
  name: String
}
```
