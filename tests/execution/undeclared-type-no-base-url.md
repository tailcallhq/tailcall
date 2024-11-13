---
error: true
---

# undeclared-type-no-base-url

```graphql @schema
schema {
  query: Query
}

type Query {
  users: [User] @http(url: "http://jsonplaceholder.typicode.com/users")
}
```
