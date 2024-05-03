---
expect_validation_error: true
---

# undeclared-type-no-base-url

```graphql @server
schema {
  query: Query
}

type Query {
  users: [User] @http(path: "/users")
}
```
