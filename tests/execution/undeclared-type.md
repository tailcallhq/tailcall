---
error: true
---

# undeclared-type

```graphql @config
schema @server {
  query: Query
}

type Query {
  users: [User] @http(baseURL: "https://jsonplaceholder.typicode.com", path: "/users")
}
```
