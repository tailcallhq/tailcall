---
error: true
---

# undeclared-type

```graphql @schema
schema @server {
  query: Query
}

type Query {
  users: [User] @http(url: "http://jsonplaceholder.typicode.com/users")
}
```
