---
expect_validation_error: true
---

# test-missing-query-resolver

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
