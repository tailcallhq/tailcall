---
expect_validation_error: true
---

# undeclared-type

```graphql @server
schema {
  query: Query
}

type Query {
  users: [User] @http(baseURL: "https://jsonplaceholder.typicode.com", path: "/users")
}
```
