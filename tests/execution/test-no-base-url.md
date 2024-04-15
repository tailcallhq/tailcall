---
expect_validation_error: true
---

# test-no-base-url

```graphql @server
schema {
  query: Query
}

type Query {
  user: User @http(path: "/user/1")
}

type User {
  id: ID!
}
```
