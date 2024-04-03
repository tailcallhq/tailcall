---
expect_validation_error: true
---

# test-expr-error

```graphql @server
schema @server @upstream(baseURL: "https://jsonplaceholder.typicode.com") {
  query: Query
}

type User {
  name: String
  age: Int!
}

type Query {
  user: User @expr(body: {name: "John"})
}
```
