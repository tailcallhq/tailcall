---
error: true
---

# test-missing-query-resolver

```graphql @schema
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
