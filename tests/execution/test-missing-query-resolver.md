# test-missing-query-resolver

---
expect_validation_error: true
---

```graphql @server
schema {
  query: Query
}

type Query {
  user: [User]
  posts: [Post]!
}

type User {
  id: ID
  name: String
}

type Post {
  id: ID
}
```
