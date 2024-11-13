---
error: true
---

# test-lack-resolver

```graphql @schema
schema {
  query: Query
}

type Query {
  posts: InPost
}

type InPost {
  get: [Post]
}

type Post {
  id: Int!
  userId: Int!
  title: String!
  body: String!
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/1")
}

type User {
  name: String
  id: Int
}
```
