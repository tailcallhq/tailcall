---
error: true
---

# test-no-base-url

```graphql @config
schema {
  query: Query
}

type User {
  id: ID!
}

type Query {
  user: User @http(path: "/user/1")
}
```
