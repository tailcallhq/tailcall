---
error: true
---

# test-no-base-url

```graphql @schema
schema {
  query: Query
}

type User {
  id: ID!
}

type Query {
  user: User @http()
}
```
