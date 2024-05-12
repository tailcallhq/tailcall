---
error: true
---

# undeclared-type-no-base-url

```graphql @server
schema @server {
  query: Query
}

type Query {
  users: [User] @http(path: "/users")
}
```
