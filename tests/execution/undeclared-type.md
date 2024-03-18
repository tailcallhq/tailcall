# undeclared-type

---
expect_validation_error: true
---

```graphql @server
schema @server {
  query: Query
}

type Query {
  users: [User] @http(baseURL: "https://jsonplaceholder.typicode.com", path: "/users")
}
```
