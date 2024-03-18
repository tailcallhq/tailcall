# test-no-base-url

---
expect_validation_error: true
---

```graphql @server
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
