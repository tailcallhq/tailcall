---
error: true
---

# test-missing-query-resolver

```graphql @config
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
