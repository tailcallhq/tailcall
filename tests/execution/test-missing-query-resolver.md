---
expect_validation_error: true
---

# test-missing-query-resolver

```graphql @server
schema {
  query: Query
}

type Post {
  id: ID
}

type Query {
  posts: [Post]!
  user: [User]
}

type User {
  id: ID
  name: String
}
```
